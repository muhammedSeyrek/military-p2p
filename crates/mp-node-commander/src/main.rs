use anyhow::Result;
use clap::{Parser, Subcommand};

mod init;
mod read;
mod serve;

#[derive(Parser)]
#[command(name = "mp-node-commander")]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// This commander's own DB connection URL
    #[arg(long, env = "COMMANDER_DB_URL")]
    db_url: String,

    /// General Staff DB URL (used during init to pull the peer directory)
    #[arg(
        long,
        env = "GENERAL_DB_URL",
        default_value = "postgresql://mp:mp_dev_pass@pg-general/general"
    )]
    general_db_url: String,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize DB: write self profile and populate peer directory
    Init {
        /// This commander's email (looked up in the General DB)
        #[arg(long)]
        email: String,
        /// Path to the PEM-encoded private key file
        #[arg(long)]
        private_key_file: String,
    },

    /// Start the HTTP server
    Serve {
        #[arg(long, default_value = "8443")]
        port: u16,
    },

    /// Read an operation: fetch parts from peers, verify Merkle, decrypt
    Read {
        #[arg(long)]
        operation: String,
    },

    /// List received operations
    ListOperations,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .init();

    let cli = Cli::parse();
    let pool = mp_storage::pool::create(&cli.db_url).await?;

    match cli.command {
        Commands::Init {
            email,
            private_key_file,
        } => {
            init::run(&pool, &cli.general_db_url, &email, &private_key_file).await?;
        }
        Commands::Serve { port } => {
            serve::run(pool, port).await?;
        }
        Commands::Read { operation } => {
            read::run(&pool, &operation).await?;
        }
        Commands::ListOperations => {
            read::list(&pool).await?;
        }
    }

    Ok(())
}
