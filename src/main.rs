use clap::{Parser, Subcommand, ValueEnum};

use crate::anime_api_data::ListType;

mod anilist_api;
mod anime_api_data;
mod db;
mod operations;

#[derive(Parser)]
#[command(version)]
struct Args {
    #[arg(short, long, global = true)]
    verbose: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Add {
        name: Option<String>,
    },
    List {
        status: Option<anime_api_data::ListType>,
    },
    Current,
    Episode,
    Cards {
        card_command: CardsCommand,
        number: u32,

        #[arg(long)]
        name: Option<String>,
    },
}

#[derive(ValueEnum, Clone)]
enum CardsCommand {
    Add,
    Total,
}

#[tokio::main]
async fn main() {
    let conn = db::connect("list.db3");
    let args = Args::parse();

    match &args.command {
        Commands::Add { name } => match name {
            Some(name) => {
                operations::add(&conn, Some(name.to_owned())).await;
            }
            None => {
                operations::add(&conn, None).await;
            }
        },

        Commands::List { status } => match status {
            Some(status) => {
                operations::list(&conn, status.to_owned());
            }
            None => {
                operations::list(&conn, ListType::All);
            }
        },
        Commands::Current => {
            operations::get_current(&conn);
        }
        Commands::Episode => {
            operations::get_episode(&conn);
        }
        Commands::Cards {
            card_command,
            number,
            name,
        } => {
            operations::add_card(&conn, card_command, number, name);
        }
    }
}
