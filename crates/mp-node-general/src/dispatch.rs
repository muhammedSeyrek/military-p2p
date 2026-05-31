//! Multi-recipient mesaj dağıtım orkestrasyonu.
//!
//! Her komutan için AYRI bir mesaj kabul eder. Mesajlar
//! `mp_protocol::multi_message::pack` ile concat edilir,
//! AES-GCM ile şifrelenir, RAID-0 ile parçalanır.

use anyhow::{anyhow, Result};
use uuid::Uuid;

use mp_crypto::aes::{self, AesKey};
use mp_crypto::hash::sha256;
use mp_crypto::merkle::MerkleTree;
use mp_crypto::rsa_keys::{self, encrypt_with_public};
use mp_network::Client;
use mp_protocol::api::DispatchOperationRequest;
use mp_protocol::multi_message;
use mp_protocol::{Commander, KeyEnvelope, Operation, OperationPart};
use mp_storage::general::{CommanderRepository, OperationRecipient, OperationRepository};
use mp_storage::PgPool;

/// (email, message) çiftleri ile dispatch et.
pub async fn run(
    pool: &PgPool,
    operation_name: &str,
    recipient_messages: &[(String, String)],
) -> Result<()> {
    if recipient_messages.is_empty() {
        return Err(anyhow!("No recipient messages"));
    }

    // 1. Tüm komutanları DB'den çek (email sırasına göre)
    let cmd_repo = CommanderRepository::new(pool.clone());
    let mut recipients: Vec<Commander> = Vec::new();
    let mut messages: Vec<Vec<u8>> = Vec::new();
    for (email, msg) in recipient_messages {
        let c = cmd_repo
            .find_by_email(email)
            .await?
            .ok_or_else(|| anyhow!("Commander not found: {}", email))?;
        recipients.push(c);
        messages.push(msg.as_bytes().to_vec());
    }
    tracing::info!(n = recipients.len(), "Recipients resolved");

    // 2. Multi-message pack et
    let msg_refs: Vec<&[u8]> = messages.iter().map(|m| m.as_slice()).collect();
    let packed = multi_message::pack(&msg_refs);
    tracing::info!(packed_len = packed.len(), "Messages packed");

    // 3. AES-GCM ile şifrele
    let aes_key = AesKey::random();
    let ciphertext =
        aes::encrypt(&aes_key, &packed).map_err(|e| anyhow!("AES encrypt failed: {}", e))?;
    let combined = ciphertext.to_bytes();

    // 4. RAID-0 ile parçala
    let parts_bytes = mp_protocol::raid::split(&combined, recipients.len())?;

    // 5. Merkle ağacı kur
    let tree =
        MerkleTree::from_chunks(&parts_bytes).map_err(|e| anyhow!("Merkle build failed: {}", e))?;
    let merkle_root = tree.root();
    tracing::info!(root = %hex::encode(merkle_root), "Merkle root computed");

    let op = Operation::new(operation_name.into(), merkle_root, recipients.len());
    let op_id = op.id;
    // AES key + nonce: birleştirilip RSA ile sarmalanacak
    let mut key_plus_nonce = Vec::with_capacity(32 + 12);
    key_plus_nonce.extend_from_slice(aes_key.as_bytes());
    key_plus_nonce.extend_from_slice(&ciphertext.nonce);

    let mut recipient_records: Vec<OperationRecipient> = Vec::new();
    let mut tasks = Vec::new();

    // 6. Her komutan için paket hazırla, paralel POST at
    for (i, cmd) in recipients.iter().enumerate() {
        let leaf_hash = sha256(&parts_bytes[i]);

        let pubkey = rsa_keys::public_from_pem(&cmd.public_key_pem)
            .map_err(|e| anyhow!("Parse pubkey for {}: {}", cmd.email, e))?;
        let encrypted_blob = encrypt_with_public(&pubkey, &key_plus_nonce)
            .map_err(|e| anyhow!("RSA encrypt for {}: {}", cmd.email, e))?;

        let req = DispatchOperationRequest {
            operation_id: op_id,
            operation_name: operation_name.into(),
            total_parts: recipients.len(),
            merkle_root_hex: hex::encode(merkle_root),
            leaf_hash_hex: hex::encode(leaf_hash),
            part_index: i,
            key_envelope: KeyEnvelope { encrypted_blob },
            part: OperationPart::new(op_id, i, parts_bytes[i].clone()),
        };

        recipient_records.push(OperationRecipient {
            id: Uuid::new_v4(),
            operation_id: op_id,
            commander_id: cmd.id,
            leaf_hash,
            part_index: i as i32,
        });

        let addr = cmd.network_address.clone();
        let email = cmd.email.clone();
        let req_clone = req.clone();
        tasks.push(tokio::spawn(async move {
            let client = match Client::new() {
                Ok(c) => c,
                Err(e) => return (email, Err(e)),
            };
            let result = client.dispatch_operation(&addr, &req_clone).await;
            (email, result)
        }));
    }

    // 7. Sonuçları topla
    let mut success_count = 0;
    for task in tasks {
        let (email, result) = task.await?;
        match result {
            Ok(resp) => {
                tracing::info!(email = %email, accepted = resp.accepted, "Dispatched");
                if resp.accepted {
                    success_count += 1;
                }
            }
            Err(e) => {
                tracing::warn!(email = %email, error = %e, "Dispatch failed");
            }
        }
    }

    // 8. Genelkurmay DB'sine sadece metadata yaz
    let op_repo = OperationRepository::new(pool.clone());
    let creator = recipients[0].id;
    op_repo.create(&op, creator).await?;
    op_repo.add_recipients(&recipient_records).await?;

    println!("\n✓ Operation '{}' dispatched", operation_name);
    println!("  Operation ID:  {}", op_id);
    println!("  Merkle root:   {}", hex::encode(merkle_root));
    println!("  Total parts:   {}", recipients.len());
    println!("  Delivered to:  {}/{}", success_count, recipients.len());
    println!();
    println!("  Per-recipient messages:");
    for (i, (email, msg)) in recipient_messages.iter().enumerate() {
        println!("    [{}] {} → \"{}\"", i, email, truncate(msg, 50));
    }

    Ok(())
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        let truncated: String = s.chars().take(max).collect();
        format!("{}...", truncated)
    }
}
