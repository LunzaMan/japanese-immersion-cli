use std::path;

use clap::{Parser, Subcommand, ValueEnum};
use rusqlite::Connection;

use crate::{
    anime_api_data::ListType,
    error_ctrl::{InvalidArgError, invalid_arg_error},
    operations::set_completed,
};
mod anilist_api;
mod anime_api_data;
mod db;
mod error_ctrl;
mod operations;
mod output;
mod utils;

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
    SetDate {
        date_type: DateType,
        date: String,
        name: String,
    },
    SetStatus {
        watch_status: anime_api_data::WatchStatus,
        name: Option<String>,
    },
    Complete {
        name: Option<String>,
        #[arg(long)]
        date: Option<String>,
    },
    Export {
        #[arg(long)]
        path: Option<path::PathBuf>,
    },
    Test,
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

#[derive(ValueEnum, Clone)]
pub enum DateType {
    Start,
    End,
}

#[tokio::main]
async fn main() {
    let file_path = utils::path_initialization();
    let conn = db::connect(file_path);
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
            verify_name_exists(&conn, name.as_deref());
            operations::add_card(&conn, card_command, number, &name.as_deref());
        }

        Commands::Episode {
            episode_mutation_type,
            name,
        } => {
            verify_name_exists(&conn, name.as_deref());
            match episode_mutation_type {
                EpisodeMutation::Set { number } => {
                    operations::update_episode_count(
                        &conn,
                        episode_mutation_type,
                        &name.as_deref(),
                        Some(number.to_owned()),
                    );
                }
                _ => {
                    operations::update_episode_count(
                        &conn,
                        episode_mutation_type,
                        &name.as_deref(),
                        None,
                    );
                }
            }
        }
        Commands::SetCurrent { name } => {
            verify_name_exists(&conn, Some(name));
            operations::set_current(&conn, name.as_str());
        }

        Commands::SetDate {
            date_type,
            date,
            name,
        } => {
            verify_name_exists(&conn, Some(name));
            operations::set_date(&conn, name, date, date_type.to_owned());
        }

        Commands::SetStatus { watch_status, name } => {
            verify_name_exists(&conn, name.as_deref());
            operations::set_watch_status(&conn, watch_status, name.as_deref());
        }

        Commands::Complete { name, date } => {
            verify_name_exists(&conn, name.as_deref());
            set_completed(&conn, name.as_deref(), date.as_deref());
        }

        Commands::Export { path } => {
            utils::initialize_export_to_csv(&conn, path);
        }

        Commands::Test => {}
    }
}

fn verify_name_exists(conn: &Connection, name: Option<&str>) {
    if let Some(extracted_name) = name
        && !db::anime_by_name_exists(conn, extracted_name).unwrap()
    {
        invalid_arg_error(InvalidArgError::InvalidName);
    }
}
