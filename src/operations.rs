use std::io::{self, Write};

use rusqlite::Connection;
use serde_json::from_str;

use crate::{CardsCommand, anilist_api, anime_api_data, db};

pub async fn add(conn: &Connection, search_arg: Option<String>) {
    let mut input_text = String::new();
    let input = io::stdin();

    // todo here match could be used to make code better
    let search_value;
    let x = search_arg.unwrap_or_else(|| "".to_string().trim().to_string());
    println!("{}", x);

    if x.is_empty() {
        println!("No x");
        print!("Anime Name: ");
        let _ = io::stdout().flush();
        input
            .read_line(&mut input_text)
            .expect("Failed to read line");

        search_value = &input_text;
    } else {
        println!("Yes x");
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
    let anime = db::query_current(conn).unwrap();

    println!("{}. {} ({})", anime.id, anime.title.english, anime.url);
}

pub fn get_episode(conn: &Connection) {
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
        "{}. {} {}/{}",
        anime.id, anime.title.english, current_episode, anime.episodes
    );
}

pub fn add_card(
    conn: &Connection,
    add_type: &CardsCommand,
    number_of_cards: &u32,
    name: &Option<String>,
) {
    let _ = db::add_card_mutation(conn, add_type, number_of_cards, name);

    let anime;
    match name {
        Some(name) => anime = db::anime_query_by_name(conn, name.to_owned()).unwrap(),
        None => anime = db::query_current(conn).unwrap(),
    };

    let db_number = anime.anki_flashcards;
    let updated_number;
    match db_number {
        Some(db_number) => updated_number = db_number,
        None => updated_number = 0,
    }
    println!("{} {} {}", anime.id, anime.title.english, updated_number);
}
