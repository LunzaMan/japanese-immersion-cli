use clap::{Parser, Subcommand, ValueEnum};
use heck::ToTitleCase;

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
    Cards {
        card_command: CardsCommand,
        number: u32,

        #[arg(long)]
        name: Option<String>,
    },
    Episode {
        #[command(subcommand)]
        episode_mutation_type: EpisodeMutation,
        #[arg(long, global = true)]
        name: Option<String>,
    },
    SetCurrent {
        name: String,
    },
}

#[derive(Subcommand)]
enum EpisodeMutation {
    Add,
    Subtract,
    Set { number: u16 },
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

        Commands::Cards {
            card_command,
            number,
            name,
        } => {
            operations::add_card(&conn, card_command, number, &name.as_deref());
        }

        Commands::Episode {
            episode_mutation_type,
            name,
        } => {
            let err;
            match episode_mutation_type {
                EpisodeMutation::Set { number } => {
                    err = operations::update_episode_count(
                        &conn,
                        episode_mutation_type,
                        &name.as_deref(),
                        Some(number.to_owned()),
                    );
                }
                _ => {
                    err = operations::update_episode_count(
                        &conn,
                        episode_mutation_type,
                        &name.as_deref(),
                        None,
                    );
                }
            }

            if let Err(err) = err {
                eprintln!("{err}");
                std::process::exit(1);
            }
        }

        Commands::SetCurrent { name } => {
            let name = name.to_title_case();
            operations::set_current(&conn, name.as_str());
        }
    }

    println!("Main gets again");
}
