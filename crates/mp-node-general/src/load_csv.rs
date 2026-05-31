//! CSV'den komutan + birlik yükle. Her komutan için yeni RSA key üretir,
//! private key'i bir dosyaya yazar (init.rs'teki gibi).

use anyhow::{anyhow, Result};
use std::path::Path;
use std::fs;
use serde::Deserialize;

use mp_storage::PgPool;
use mp_storage::general::{CommanderRepository, UnitRepository};
use mp_storage::migrate_general;
use mp_protocol::{Commander, Rank, Unit};
use mp_crypto::rsa_keys::KeyPair;

#[derive(Debug, Deserialize)]
struct CsvRow {
    corps_number: i32,
    unit_name: String,
    unit_type: String,
    location: String,
    full_name: String,
    email: String,
    rank: String,
}

pub async fn run(pool: &PgPool, csv_path: &str, keys_dir: &str) -> Result<()> {
    tracing::info!("Running migrations...");
    migrate_general(pool).await?;

    // Keys klasörünü oluştur
    fs::create_dir_all(keys_dir)
        .map_err(|e| anyhow!("Cannot create keys dir '{}': {}", keys_dir, e))?;

    // CSV oku
    let path = Path::new(csv_path);
    if !path.exists() {
        return Err(anyhow!("CSV file not found: {}", csv_path));
    }
    let mut rdr = csv::Reader::from_path(path)?;

    let unit_repo = UnitRepository::new(pool.clone());
    let cmd_repo = CommanderRepository::new(pool.clone());

    // Zaten kayıtlı mı kontrol
    let existing = cmd_repo.list_all().await?;
    if !existing.is_empty() {
        return Err(anyhow!(
            "DB already has {} commanders. Run with fresh DB to load CSV.",
            existing.len()
        ));
    }

    let mut count = 0;
    for result in rdr.deserialize() {
        let row: CsvRow = result.map_err(|e| anyhow!("CSV parse: {}", e))?;
        let rank = Rank::from_str(&row.rank)?;

        // Birlik oluştur
        let unit = Unit::new(
            row.corps_number,
            row.unit_name.clone(),
            row.unit_type.clone(),
            row.location.clone(),
        );
        unit_repo.create(&unit).await?;

        // Yeni RSA key çifti üret
        let kp = KeyPair::generate()?;
        let pub_pem = kp.public_to_pem()?;
        let priv_pem = kp.private_to_pem()?;

        // Address: ad'dan slug üret
        let slug = slugify(&row.full_name);
        let network_address = format!("http://node-{}:8443", slug);

        let mut cmd = Commander::new(
            row.full_name.clone(),
            row.email.clone(),
            rank,
            pub_pem,
            network_address.clone(),
        );
        cmd.unit_id = Some(unit.id);
        cmd_repo.create(&cmd).await?;

        // Private key'i dosyaya yaz
        let key_path = format!("{}/key-{}.pem", keys_dir.trim_end_matches('/'), slug);
        fs::write(&key_path, &priv_pem)
            .map_err(|e| anyhow!("Cannot write {}: {}", key_path, e))?;

        println!("✓ {:<20} {:<12} {:<25} → {}",
                 row.full_name, rank.as_str(), row.location, key_path);
        count += 1;
    }

    println!("\n✓ Loaded {} commanders from CSV", count);
    println!("  Keys directory: {}", keys_dir);
    println!("  Use these private key files when initializing commander nodes.");
    Ok(())
}

/// "Aylin Kaya" → "aylin", "Mehmet Yilmaz" → "mehmet"
/// İlk kelimeyi alır, küçük harfe çevirir, Türkçe karakterleri normalize eder.
fn slugify(name: &str) -> String {
    let first = name.split_whitespace().next().unwrap_or("");
    first.chars()
        .map(|c| match c {
            'ç' | 'Ç' => 'c',
            'ğ' | 'Ğ' => 'g',
            'ı' => 'i', 'I' => 'i',
            'İ' => 'i', 'i' => 'i',
            'ö' | 'Ö' => 'o',
            'ş' | 'Ş' => 's',
            'ü' | 'Ü' => 'u',
            c => c.to_ascii_lowercase(),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slugify_turkish() {
        assert_eq!(slugify("Aylin Kaya"), "aylin");
        assert_eq!(slugify("Şule Yılmaz"), "sule");
        assert_eq!(slugify("Mehmet Yilmaz"), "mehmet");
        assert_eq!(slugify("Ömer Çelik"), "omer");
    }
}
