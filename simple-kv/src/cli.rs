use clap::{Parser, Subcommand};

use crate::cli::Commands::{Clean, Keys};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Put {
        key: String,
        value: String,
    },
    Get {
        key: String,
    },
    Remove {
        key: String,
    },
    /// Show all the keys and print them to the stdout
    Keys,
    Clean,
}

pub fn run_command() {
    let cli = Cli::parse();

    // You can check for the existence of subcommands, and if found use their
    // matches just as you would the top level cmd
    match cli.command {
        Commands::Put { key, value } => {
            let mut db = crate::db::Database::new();
            db.put(key, value);
        }
        Commands::Get { key } => {
            let db = crate::db::Database::new();
            db.get(&key);
        }
        Commands::Remove { key } => {
            let mut db = crate::db::Database::new();
            db.remove(&key);
        }
        Keys => {
            let db = crate::db::Database::new();
            db.keys();
        }
        Clean => {
            let mut db = crate::db::Database::new();
            db.clean();
        }
    }
}
