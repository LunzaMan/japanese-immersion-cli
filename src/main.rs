use serde::{Deserialize, Serialize};
use serde_json::from_str;
use std::io;

mod anilist_api;

#[derive(Serialize, Deserialize, Debug)]
struct Anime {
    id: usize,
    episodes: u8,
    title: Title,
    #[serde(rename = "type")]
    media_type: String,
    #[serde(rename = "siteUrl")]
    url: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Title {
    english: String,
    native: String,
    romaji: String,
}

async fn search() {
    let mut search_value = String::new();

    // todo: make the input in the same line
    println!("Anime Name:");
    io::stdin()
        .read_line(&mut search_value)
        .expect("Failed to read line");

    // todo: expriment: try to send ony the result array from use_api
    let result = anilist_api::browse(search_value.to_string()).await;
    let result_arr = &result["data"]["Page"]["media"].as_array().unwrap();

    for i in 0..result_arr.len() {
        println!("{}. {}", i + 1, result_arr[i]["title"]["romaji"]);
    }

    // todo: add a quit function

    let mut choice = String::new();

    io::stdin()
        .read_line(&mut choice)
        .expect("Failed to read line");

    let x: usize = choice.trim().parse().expect("Input is not a number");

    let id = &result_arr[x - 1]["id"].as_number().unwrap();
    let anime_json = anilist_api::get_by_id(id).await["data"]["Media"].to_string();

    let anime = from_str::<Anime>(&anime_json).expect("Could create object");

    println!("{:#} {:}", anime.id, anime.title.english);

    // Todo: next step = add the data struct to sqlite
}

#[tokio::main]
async fn main() {
    search().await;
}
