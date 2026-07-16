use std::io::{self, Write};

use heck::ToTitleCase;
use rusqlite::Connection;
use serde_json::from_str;

use crate::{
    CardsCommand, DateType, EpisodeMutation, anilist_api,
    anime_api_data::{self, WatchStatus},
    db::{self},
    error_ctrl::{self, InvalidArgError, invalid_arg_error},
    utils,
};

pub async fn add(conn: &Connection, search_arg: Option<String>) {
    let mut input_text = String::new();
    let input = io::stdin();

    // todo here match could be used to make code better
    let search_value;
    let x = search_arg.unwrap_or_else(|| "".to_string().trim().to_string());

    if x.is_empty() {
        print!("Anime Name: ");
        let _ = io::stdout().flush();
        input
            .read_line(&mut input_text)
            .expect("Failed to read line");

        search_value = &input_text;
    } else {
        search_value = &x;
    }

    // todo: expriment: try to send ony the result array from use_api
    // todo: fix the infinite running of api if title not found.
    let result = anilist_api::browse(search_value.to_string()).await;
    let result_arr = &result["data"]["Page"]["media"].as_array().unwrap();

    for i in 0..result_arr.len() {
        println!("{}. {}", i + 1, result_arr[i]["title"]["romaji"]);
    }

    // todo: add a quit function

    input_text.clear();
    input
        .read_line(&mut input_text)
        .expect("Failed to read line");

    let x: usize = input_text.trim().parse().expect("Input is not a number");

    let id = &result_arr[x - 1]["id"].as_number().unwrap();
    let anime_json = anilist_api::get_by_id(id).await["data"]["Media"].to_string();

    // todo: use only one api call instead of 2 to increase speed
    let anime = from_str::<anime_api_data::Anime>(&anime_json).expect("Could create object");

    println!(
        "Selected Anime: {}, Link:{}",
        anime.get_english_title(),
        anime.get_url()
    );

    let watch_status: anime_api_data::WatchStatus;
    loop {
        println!("Set Watch Status: \n1. Watching\n2. Planning\n3. Completed");

        input_text.clear();
        input
            .read_line(&mut input_text)
            .expect("Failed to read line");

        // Todo: When no input is given default to watching
        // Todo: If not number then keep running. Also add a quit command
        let watch_status_input: u8 = input_text.trim().parse().expect("Failed to parse input");

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
            _ => {
                println!("Choose correct option");
                anime_api_data::WatchStatus::Planning
            }
        };
    }

    //todo: If they don't choose watching then don't show this message
    println!("Set anime as currently watching?(y/N)");
    println!("If set to currently watching then date started will be set to today");
    input_text.clear();
    input
        .read_line(&mut input_text)
        .expect("Failed to readline");

    let choice: char = input_text.remove(0);

    let mut _is_current = false;

    match choice {
        'y' => {
            _is_current = true;
        }
        _ => {
            _is_current = false;
        }
    }

    _ = db::add_anime(conn, anime, watch_status, _is_current);
}

pub fn list(conn: &Connection, list_type: anime_api_data::ListType) {
    let result =
        db::query_list(conn, list_type.to_owned(), anime_api_data::MediaType::Anime).unwrap();

    if result.is_empty() {
        println!("No {} anime in database ", list_type);
    } else {
        let result_iter = result.iter();

        for anime in result_iter {
            println!("{}. {} ({})", anime.id, anime.title.english, anime.url);
        }
    }
}

pub fn get_current(conn: &Connection) {
    if db::current_exists(conn).unwrap() {
        let anime = db::query_current(conn).unwrap();

        let current_episode: u16;

        let x = anime.episode_progress;
        match x {
            Some(x) => current_episode = x,
            None => {
                current_episode = 0;
            }
        }
        println!(
            "{}. {} {}/{} ({})",
            anime.id, anime.title.english, current_episode, anime.episodes, anime.url
        );
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

    let db_number = anime.anki_flashcards;
    let updated_number;
    match db_number {
        Some(db_number) => updated_number = db_number,
        None => updated_number = 0,
    }
    println!("{} {} {}", anime.id, anime.title.english, updated_number);
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

    let anime_name = name.to_title_case();
    let selected_anime = get_anime_from_option(conn, &Some(&anime_name.as_str()));

    let is_planning = match selected_anime.watch_status.unwrap() {
        anime_api_data::WatchStatus::Planning => true,
        _ => false,
    };

    if is_prev_current {
        let prev_current_anime = db::query_current(conn).unwrap();

        if prev_current_anime.id == selected_anime.id {
            println!("{} was already current anime", selected_anime.title.english);

            if is_planning {
                let _ = db::date_mutation(conn, name, None, DateType::Start);
            }

            return;
        }

        let _ = db::remove_current(conn).unwrap();
    }

    let _ = db::set_current(conn, name).unwrap();

    if is_planning {
        let _ = db::date_mutation(conn, name, None, DateType::Start);
    }

    let anime = get_anime_from_option(conn, &None);

    println!("Current anime:");
    println!(
        "{}. {} {}/{}",
        anime.id,
        anime.title.english,
        anime.episode_progress.unwrap(),
        anime.episodes
    );
}

pub fn set_date(conn: &Connection, name: &str, date: &str, date_type: DateType) {
    let _ = db::date_mutation(conn, name, Some(date), date_type);

    let anime = db::anime_query_by_name(conn, name).unwrap();

    println!(
        "{}. {} {}/{} | start-date: {}",
        anime.id,
        anime.title.english,
        anime.episode_progress.unwrap_or_else(|| 0),
        anime.episodes,
        anime.date_started.unwrap_or_else(|| "None".to_string())
    );
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

    println!(
        "{}. {} {}/{} | start-date: {}",
        anime.id,
        anime.title.english,
        anime.episode_progress.unwrap(),
        anime.episodes,
        anime.watch_status.unwrap()
    );
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
            let _ = db::date_mutation(conn, anime.title.english.as_str(), date, DateType::End);
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
                        db::date_mutation(conn, anime.title.english.as_str(), date, DateType::End);
                }
                _ => {}
            }
        }
    }

    if anime.is_current.unwrap() == true {
        let _ = db::remove_current(conn);
    }
}
