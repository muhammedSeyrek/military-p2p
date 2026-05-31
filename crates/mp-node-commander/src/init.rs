//! Commander DB init: write self profile and populate the peer directory.

use anyhow::{anyhow, Result};
use chrono::Utc;
use mp_protocol::Rank;
use mp_storage::commander::{Peer, PeerRepository, SelfInfo, SelfInfoRepository};
use mp_storage::general::CommanderRepository;
use mp_storage::{migrate_commander, pool, PgPool};
use uuid::Uuid;

pub async fn run(
    pool: &PgPool,
    general_db_url: &str,
    email: &str,
    private_key_file: &str,
) -> Result<()> {
    tracing::info!("Running commander migrations...");
    migrate_commander(pool).await?;

    // Read private key from file.
    let private_key_pem = std::fs::read_to_string(private_key_file)
        .map_err(|e| anyhow!("Cannot read private key file '{}': {}", private_key_file, e))?;

    // Fetch the full commander directory from the General DB.
    let general_pool = pool::create(general_db_url).await?;
    let cmd_repo = CommanderRepository::new(general_pool);
    let all_commanders = cmd_repo.list_all().await?;

    // Find this commander by email.
    let me = all_commanders
        .iter()
        .find(|c| c.email == email)
        .ok_or_else(|| anyhow!("Email '{}' not found in general directory", email))?;

    // Write self_info.
    let self_repo = SelfInfoRepository::new(pool.clone());
    let existing = self_repo.get().await?;
    if existing.is_some() {
        tracing::info!("Self info already exists, skipping");
    } else {
        let info = SelfInfo {
            id: Uuid::new_v4(),
            commander_id: me.id,
            full_name: me.full_name.clone(),
            email: me.email.clone(),
            rank: me.rank,
            password_hash: "$argon2id$disabled$dev".into(),
            private_key_pem,
            created_at: Utc::now(),
        };
        self_repo.create(&info).await?;
        tracing::info!(name = %me.full_name, "Self info written");
    }

    // Populate peer_directory with the other commanders.
    let peer_repo = PeerRepository::new(pool.clone());
    let mut peer_count = 0;
    for c in &all_commanders {
        if c.id == me.id {
            continue; // Don't add ourselves as a peer.
        }
        let peer = Peer {
            commander_id: c.id,
            full_name: c.full_name.clone(),
            email: c.email.clone(),
            rank: c.rank,
            public_key_pem: c.public_key_pem.clone(),
            network_address: c.network_address.clone(),
            created_at: Utc::now(),
        };
        peer_repo.upsert(&peer).await?;
        peer_count += 1;
    }

    println!("✓ Initialized commander '{}'", me.full_name);
    println!("  Commander ID: {}", me.id);
    println!("  Rank: {} ({})", me.rank.as_str(), rank_level(me.rank));
    println!("  Peers loaded: {}", peer_count);
    Ok(())
}

fn rank_level(r: Rank) -> &'static str {
    match r {
        Rank::Maresal => "1/6",
        Rank::Orgeneral => "2/6",
        Rank::Korgeneral => "3/6",
        Rank::Tumgeneral => "4/6",
        Rank::Tuggeneral => "5/6",
        Rank::Albay => "6/6",
    }
}
