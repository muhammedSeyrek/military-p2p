use clap::{Parser, Subcommand};
use anyhow::{anyhow, Result};

mod dispatch;
mod init;
mod load_csv;

#[derive(Parser)]
#[command(name = "mp-node-general")]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[arg(long, env = "GENERAL_DB_URL",
          default_value = "postgresql://mp:mp_dev_pass@pg-general/general")]
    db_url: String,
}

#[derive(Subcommand)]
enum Commands {
    Init,

    LoadCsv {
        #[arg(long, default_value = "data/commanders.csv")]
        file: String,
        #[arg(long, default_value = "/tmp/keys")]
        keys_dir: String,
    },

    ListCommanders,

    /// Her komutan için ayrı mesajla dispatch.
    ///
    /// Örnek:
    ///   --to "mehmet@...:Doğuya intikal et" \
    ///   --to "ali@...:Erzak ikmali sağla"
    Dispatch {
        #[arg(long)]
        name: String,

        /// email:mesaj formatı, her komutan için bir tane
        #[arg(long = "to", value_name = "EMAIL:MESSAGE")]
        recipient_messages: Vec<String>,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .init();

    let cli = Cli::parse();
    let pool = mp_storage::pool::create(&cli.db_url).await?;

    match cli.command {
        Commands::Init => init::run(&pool).await?,
        Commands::LoadCsv { file, keys_dir } => {
            load_csv::run(&pool, &file, &keys_dir).await?;
        }
        Commands::ListCommanders => init::list(&pool).await?,
        Commands::Dispatch { name, recipient_messages } => {
            let parsed = parse_recipient_messages(&recipient_messages)?;
            dispatch::run(&pool, &name, &parsed).await?;
        }
    }

    Ok(())
}

/// "email:mesaj" stringlerini (email, mesaj) tuple'larına çevir.
fn parse_recipient_messages(raw: &[String]) -> Result<Vec<(String, String)>> {
    if raw.is_empty() {
        return Err(anyhow!("No --to arguments provided"));
    }
    let mut result = Vec::new();
    for s in raw {
        let (email, msg) = s.split_once(':')
            .ok_or_else(|| anyhow!("Invalid --to format (need email:message): {}", s))?;
        result.push((email.trim().to_string(), msg.trim().to_string()));
    }
    Ok(result)
}
