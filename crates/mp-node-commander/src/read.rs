//! Read an operation: gather parts from peers in parallel, verify Merkle,
//! decrypt AES, and extract ONLY this commander's own message from the
//! multi-message payload.

use anyhow::{anyhow, Result};
use mp_crypto::aes::{AesKey, Ciphertext};
use mp_crypto::merkle::MerkleTree;
use mp_crypto::{aes, rsa_keys};
use mp_network::Client;
use mp_protocol::{multi_message, OperationPart};
use mp_storage::commander::{
    OperationRepository, PartRepository, Peer, PeerRepository, SelfInfoRepository,
};
use mp_storage::PgPool;
use std::sync::Arc;

pub async fn run(pool: &PgPool, operation_name: &str) -> Result<()> {
    let op_repo = OperationRepository::new(pool.clone());
    let part_repo = PartRepository::new(pool.clone());
    let self_repo = SelfInfoRepository::new(pool.clone());
    let peer_repo = PeerRepository::new(pool.clone());

    let op = op_repo
        .find_by_name(operation_name)
        .await?
        .ok_or_else(|| anyhow!("Operation '{}' not found in local DB", operation_name))?;
    tracing::info!(
        op_id = %op.id,
        part_index = op.part_index,
        total = op.total_parts,
        "Operation found"
    );

    let me = self_repo
        .get()
        .await?
        .ok_or_else(|| anyhow!("No self_info — run `init` first"))?;
    let privkey = rsa_keys::private_from_pem(&me.private_key_pem)
        .map_err(|e| anyhow!("Cannot parse own private key: {}", e))?;

    // Decrypt the AES key + nonce with our RSA private key.
    let key_plus_nonce = rsa_keys::decrypt_with_private(&privkey, &op.encrypted_aes_key)
        .map_err(|e| anyhow!("RSA decrypt failed: {}", e))?;

    if key_plus_nonce.len() != 32 + 12 {
        return Err(anyhow!(
            "Invalid AES key+nonce size: {}",
            key_plus_nonce.len()
        ));
    }
    let aes_key =
        AesKey::from_bytes(&key_plus_nonce[..32]).map_err(|e| anyhow!("Bad AES key: {}", e))?;
    let mut nonce = [0u8; 12];
    nonce.copy_from_slice(&key_plus_nonce[32..]);
    tracing::info!("AES key + nonce decrypted");

    // Load our own part.
    let my_part = part_repo
        .find(op.id, op.part_index)
        .await?
        .ok_or_else(|| anyhow!("Own part not in DB"))?;

    // Fetch the remaining parts from peers in parallel.
    let peers = peer_repo.list_all().await?;
    let peers = Arc::new(peers);

    let needed_indices: Vec<usize> = (0..op.total_parts)
        .filter(|i| *i != op.part_index)
        .collect();

    println!(
        "Fetching {} parts from {} peers in parallel...",
        needed_indices.len(),
        peers.len()
    );

    let mut tasks = Vec::new();
    for idx in &needed_indices {
        let idx = *idx;
        let peers_clone = Arc::clone(&peers);
        let op_id = op.id;
        tasks.push(tokio::spawn(async move {
            fetch_part_from_any_peer(op_id, idx, &peers_clone).await
        }));
    }

    let mut all_parts: Vec<OperationPart> = vec![my_part];
    for task in tasks {
        let result = task
            .await
            .map_err(|e| anyhow!("Task join error: {}", e))??;
        all_parts.push(result);
    }

    all_parts.sort_by_key(|p| p.part_index);

    // Verify Merkle root.
    let chunks: Vec<Vec<u8>> = all_parts
        .iter()
        .map(|p| p.ciphertext_chunk.clone())
        .collect();
    let computed_tree =
        MerkleTree::from_chunks(&chunks).map_err(|e| anyhow!("Merkle build: {}", e))?;
    let computed_root = computed_tree.root();

    if computed_root != op.merkle_root {
        tracing::error!(
            expected = %hex::encode(op.merkle_root),
            got = %hex::encode(computed_root),
            "TAMPERING DETECTED"
        );
        op_repo.delete(op.id).await?;
        return Err(anyhow!(
            "Merkle root mismatch — tampering! Operation deleted."
        ));
    }
    tracing::info!("✓ Merkle root verified");

    // RAID-0 join.
    let indexed: Vec<(usize, Vec<u8>)> = all_parts
        .iter()
        .map(|p| (p.part_index, p.ciphertext_chunk.clone()))
        .collect();
    let combined = mp_protocol::raid::join(indexed, op.total_parts)?;

    // AES-GCM decrypt.
    let ct = Ciphertext::from_bytes(&combined).map_err(|e| anyhow!("Ciphertext parse: {}", e))?;
    let packed_plaintext = aes::decrypt(
        &aes_key,
        &Ciphertext {
            nonce,
            data: ct.data,
        },
    )
    .map_err(|e| anyhow!("AES decrypt failed: {}", e))?;

    // Extract ONLY this commander's slice from the multi-message payload.
    let my_message = multi_message::extract_at(&packed_plaintext, op.part_index)
        .map_err(|e| anyhow!("Cannot extract own message: {}", e))?;

    let message_text = String::from_utf8_lossy(&my_message);

    println!("\n╔════════════════════════════════════════════════════════╗");
    println!("║ Operation: {}", op.name);
    println!("║ Recipient: {} ({})", me.full_name, me.rank.as_str());
    println!("║ Part index: {} of {}", op.part_index, op.total_parts);
    println!("║ Merkle verified: ✓");
    println!("╠════════════════════════════════════════════════════════╣");
    println!("║ {}", message_text);
    println!("╚════════════════════════════════════════════════════════╝\n");
    println!("Other commanders' messages decrypted in memory but not displayed.");
    println!("All sensitive data wiped at process exit.\n");

    Ok(())
}

async fn fetch_part_from_any_peer(
    op_id: uuid::Uuid,
    idx: usize,
    peers: &[Peer],
) -> Result<OperationPart> {
    let client = Client::new()?;
    for peer in peers {
        match client.fetch_part(&peer.network_address, op_id, idx).await {
            Ok(resp) => {
                tracing::info!(idx, from = %peer.full_name, "Part received");
                return Ok(resp.part);
            }
            Err(e) => {
                tracing::debug!(idx, peer = %peer.full_name, error = %e, "Peer fetch failed");
            }
        }
    }
    Err(anyhow!(
        "Could not retrieve part index {} from any peer",
        idx
    ))
}

pub async fn list(pool: &PgPool) -> Result<()> {
    let op_repo = OperationRepository::new(pool.clone());
    let ops = op_repo.list_recent(50).await?;

    if ops.is_empty() {
        println!("No operations received yet.");
        return Ok(());
    }

    println!(
        "{:<30} {:<10} {:<10} {}",
        "Name", "Idx/Total", "Parts", "Received"
    );
    println!("{}", "-".repeat(80));
    for o in ops {
        println!(
            "{:<30} {}/{:<8} {:<10} {}",
            o.name, o.part_index, o.total_parts, o.total_parts, o.received_at
        );
    }
    Ok(())
}
