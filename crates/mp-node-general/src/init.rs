use anyhow::Result;
use mp_crypto::rsa_keys::KeyPair;
use mp_protocol::{Commander, Rank, Unit};
use mp_storage::general::{CommanderRepository, UnitRepository};
use mp_storage::migrate_general;
use mp_storage::PgPool;

pub async fn run(pool: &PgPool) -> Result<()> {
    tracing::info!("Running migrations...");
    migrate_general(pool).await?;

    let unit_repo = UnitRepository::new(pool.clone());
    let cmd_repo = CommanderRepository::new(pool.clone());

    let existing = cmd_repo.list_all().await?;
    if !existing.is_empty() {
        tracing::info!(
            count = existing.len(),
            "Commanders already exist, skipping seed"
        );
        return Ok(());
    }

    let seeds = [
        (
            "Aylin Kaya",
            "aylinkaya@karakuvvetleri.mil.tr",
            Rank::Tuggeneral,
            "http://node-aylin:8443",
            (1, "8.MekanizePiyade", "Tugayi", "Tekirdag"),
        ),
        (
            "Koray Aydin",
            "korayaydin@karakuvvetleri.mil.tr",
            Rank::Tumgeneral,
            "http://node-koray:8443",
            (2, "23.Piyade", "Tumeni", "Sirnak"),
        ),
        (
            "Emre Demir",
            "emredemir@karakuvvetleri.mil.tr",
            Rank::Orgeneral,
            "http://node-emre:8443",
            (1, "18.MekanizePiyade", "Tugayi", "Ortakoy"),
        ),
    ];

    for (full_name, email, rank, addr, (corps, unit_name, unit_type, location)) in seeds {
        let unit = Unit::new(corps, unit_name.into(), unit_type.into(), location.into());
        unit_repo.create(&unit).await?;

        let kp = KeyPair::generate()?;
        let pub_pem = kp.public_to_pem()?;
        let priv_pem = kp.private_to_pem()?;

        let mut cmd = Commander::new(full_name.into(), email.into(), rank, pub_pem, addr.into());
        cmd.unit_id = Some(unit.id);
        cmd_repo.create(&cmd).await?;

        println!("=== {} ===", full_name);
        println!("Email: {}", email);
        println!("Commander ID: {}", cmd.id);
        println!("Address: {}", addr);
        println!("PRIVATE KEY (save this):");
        println!("{}", priv_pem);
        println!();
    }

    tracing::info!("Init complete. 3 commanders seeded.");
    Ok(())
}

pub async fn list(pool: &PgPool) -> Result<()> {
    let cmd_repo = CommanderRepository::new(pool.clone());
    let all = cmd_repo.list_all().await?;

    if all.is_empty() {
        println!("No commanders. Run `init` first.");
        return Ok(());
    }

    println!(
        "{:<25} {:<35} {:<12} {}",
        "Name", "Email", "Rank", "Address"
    );
    println!("{}", "-".repeat(100));
    for c in all {
        println!(
            "{:<25} {:<35} {:<12} {}",
            c.full_name,
            c.email,
            c.rank.as_str(),
            c.network_address
        );
    }
    Ok(())
}
