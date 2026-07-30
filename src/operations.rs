use std::io::{self, Write};

use clap::builder::Str;
use rusqlite::Connection;
use serde_json::{from_str, from_value};

use crate::{
    CardsCommand, DateType, EpisodeMutation, anilist_api,
    anime_api_data::{self, WatchStatus},
    db::{self},
    error_ctrl::{self, InvalidArgError, invalid_arg_error},
    output::{self, PrintStyle},
    utils::{self, parse_browse},
};

fn get_input(display_text: Option<&str>) -> String {
    let mut input_text = String::new();

    match display_text {
        Some(display_text) => {
            print!("{}", display_text);
            io::stdout().flush().expect("Couldn't display display_text");
        }
        None => {}
    }
    io::stdin()
        .read_line(&mut input_text)
        .expect("Input can't be read");

    input_text
}

pub async fn add(conn: &Connection, search_arg: Option<String>) {
    let search_value = match search_arg {
        Some(search_arg) => search_arg,
        None => get_input(Some("Anime Name: ")),
    };

    let result = parse_browse(search_value).await;

    if result.is_empty() {
        error_ctrl::invalid_arg_error(InvalidArgError::InvalidAnime);
    }

    for (i, anime) in result.iter().enumerate() {
        println!("{}. {}", i + 1, anime.title.romaji)
    }

    let choice: usize = get_input(None)
        .trim()
        .parse()
        .expect("Input is not a number");

    let anime = result
        .get(choice - 1)
        .unwrap_or_else(|| error_ctrl::invalid_arg_error(InvalidArgError::InvalidChoice));

    println!(
        "Selected Anime: {}, Link:{}",
        anime.get_english_title(),
        anime.get_url()
    );

    let watch_status: anime_api_data::WatchStatus;
    loop {
        // Todo: When no input is given default to watching
        let watch_status_input: u8 = get_input(Some(
            "Set Watch Status: \n1. Watching\n2. Planning\n3. Completed\n4. Quit",
        ))
        .trim()
        .parse()
        .expect("Failed to parse input");

        match watch_status_input {
            1 => {
                watch_status = anime_api_data::WatchStatus::Watching;
                break;
            }
            2 => {
                watch_status = anime_api_data::WatchStatus::Planning;
                break;
            }
            3 => {
                watch_status = anime_api_data::WatchStatus::Completed;
                break;
            }
            4 => {
                println!("Terminating...");
                error_ctrl::exit_app();
            }
            _ => {
                println!("Choose correct option");
                anime_api_data::WatchStatus::Planning
            }
        };
    }

    let is_current;
    match watch_status {
        WatchStatus::Watching => {
            let choice = get_input(Some("Set anime as currently watching?(y/N)"));

            match choice.to_lowercase().as_str() {
                "y" => {
                    is_current = true;
                }
                _ => {
                    is_current = false;
                }
            }
        }
        _ => is_current = false,
    }

    _ = db::add_anime(conn, anime, watch_status, is_current);
}

pub fn list(conn: &Connection, list_type: anime_api_data::ListType) {
    let result =
        db::query_list(conn, list_type.to_owned(), anime_api_data::MediaType::Anime).unwrap();

    if result.is_empty() {
        println!("No {} anime in database ", list_type);
    } else {
        output::list_print(result);
    }
}

pub fn get_current(conn: &Connection) {
    if db::current_exists(conn).unwrap() {
        let anime = db::query_current(conn).unwrap();

        output::single_print(anime, output::PrintStyle::Current);
    } else {
        invalid_arg_error(InvalidArgError::Current);
    }
}

pub fn add_card(
    conn: &Connection,
    add_type: &CardsCommand,
    number_of_cards: &u32,
    name: &Option<&str>,
) {
    let _ = db::add_card_mutation(conn, add_type, number_of_cards, name);

    let anime = get_anime_from_option(conn, name);

    output::single_print(anime, PrintStyle::Anki);
}

fn get_anime_from_option(conn: &Connection, name: &Option<&str>) -> anime_api_data::Anime {
    let anime = match name {
        Some(name) => {
            let is_exist = db::anime_by_name_exists(conn, name).unwrap();
            if !is_exist {
                invalid_arg_error(InvalidArgError::InvalidName);
            }
            db::anime_query_by_name(conn, name).unwrap()
        }
        None => {
            if db::current_exists(conn).unwrap() {
                db::query_current(conn).unwrap()
            } else {
                invalid_arg_error(InvalidArgError::Current);
            }
        }
    };

    anime
}

pub fn update_episode_count(
    conn: &Connection,
    mut_type: &EpisodeMutation,
    name: &Option<&str>,
    number: Option<u16>,
) {
    let anime = utils::return_anime_if_exists(conn, name.as_deref());

    let already_completed;
    match mut_type {
        EpisodeMutation::Set { number } => {
            already_completed = true;
            if number > &anime.episodes {
                invalid_arg_error(InvalidArgError::InvalidEpisodeCount);
            }
        }
        _ => {
            if anime.episodes == anime.episode_progress.unwrap() {
                println!("Anime was already completed adding it to completed");
                already_completed = true;
            } else {
                already_completed = false;
            }
        }
    };

    if !already_completed {
        let _ = db::episode_mutation(conn, mut_type, name, number);
    }

    let anime = get_anime_from_option(conn, name);

    if anime.episodes == anime.episode_progress.unwrap() {
        println!("Anime Set to Completed");
        set_completed(conn, name.as_deref(), None);
    }
}

pub fn set_current(conn: &Connection, name: &str) {
    let is_prev_current = db::current_exists(conn).unwrap();

    let selected_anime = get_anime_from_option(conn, &Some(&name));

    let is_planning = match selected_anime.watch_status.unwrap() {
        anime_api_data::WatchStatus::Planning => true,
        _ => false,
    };

    if is_prev_current {
        let prev_current_anime = db::query_current(conn).unwrap();

        if prev_current_anime.id == selected_anime.id {
            println!("{} was already current anime", selected_anime.title.english);

            if is_planning {
                let _ = db::date_mutation(conn, name, None, &DateType::Start);
            }

            return;
        }

        let _ = db::remove_current(conn).unwrap();
    }

    let _ = db::set_current(conn, name).unwrap();

    if is_planning {
        let _ = db::date_mutation(conn, name, None, &DateType::Start);
    }

    let anime = get_anime_from_option(conn, &None);

    println!("Current anime:");
    output::single_print(anime, output::PrintStyle::Current);
}

pub fn set_date(conn: &Connection, name: &str, date: &str, date_type: DateType) {
    let _ = db::date_mutation(conn, name, Some(date), &date_type);

    let anime = db::anime_query_by_name(conn, name).unwrap();

    match date_type {
        DateType::Start => {
            output::single_print(anime, PrintStyle::StartDate);
        }
        DateType::End => {
            output::single_print(anime, PrintStyle::EndDate);
        }
    };
}

pub fn set_watch_status(
    conn: &Connection,
    watch_status: &anime_api_data::WatchStatus,
    name: Option<&str>,
) {
    let anime = utils::return_anime_if_exists(conn, name);

    let prev_watch_status = anime.watch_status.unwrap();
    if watch_status.to_owned() == prev_watch_status {
        println!("{} was already the status", prev_watch_status);
        error_ctrl::exit_app();
    }

    let _ = db::watch_status_mutation(conn, watch_status, name);

    let anime = utils::return_anime_if_exists(conn, name);

    output::single_print(anime, PrintStyle::WatchStatus);
}

pub fn set_completed(conn: &Connection, name: Option<&str>, date: Option<&str>) {
    let anime = utils::return_anime_if_exists(conn, name);

    let prev_watch_status = anime.watch_status.unwrap();

    match prev_watch_status {
        WatchStatus::Completed => {
            println!("{} was already set to completed", anime.title.english);
            error_ctrl::exit_app();
        }
        _ => {}
    };

    let total_episodes = anime.episodes;

    let _ = db::episode_mutation(
        conn,
        &EpisodeMutation::Set {
            number: total_episodes,
        },
        &name,
        Some(total_episodes),
    );

    let _ = db::watch_status_mutation(conn, &WatchStatus::Completed, name);

    let prev_date_completed = anime.date_completed;
    match prev_date_completed {
        None => {
            let _ = db::date_mutation(conn, anime.title.english.as_str(), date, &DateType::End);
        }
        Some(prev_date_completed) => {
            let mut user_input = String::new();

            println!(
                "Anime was completed on {}. Do you want to override(y/N)",
                prev_date_completed
            );
            std::io::stdin()
                .read_line(&mut user_input)
                .expect("Couldn't read line");

            match user_input.to_lowercase().as_str() {
                "y" => {
                    let _ =
                        db::date_mutation(conn, anime.title.english.as_str(), date, &DateType::End);
                }
                _ => {}
            }
        }
    }

    if anime.is_current.unwrap() == true {
        let _ = db::remove_current(conn);
    }
}
