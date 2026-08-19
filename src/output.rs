use colored::Colorize;

use crate::anime_api_data;

pub enum PrintStyle {
    Current,
    StartDate,
    EndDate,
    Anki,
    WatchStatus,
}

pub fn single_print(anime: anime_api_data::Anime, print_style: PrintStyle) {
    let name = anime.title.english.bright_yellow();
    let id = anime.id.to_string().bright_red();
    let current_ep = anime
        .episode_progress
        .unwrap_or(0)
        .to_string()
        .bright_green();
    let ep = anime.episodes.to_string().bright_green();
    let slash = "/".to_string().bright_green();

    let base_string = format!("{id}. {name}  {current_ep}{slash}{ep}");

    match print_style {
        PrintStyle::Current => {
            println!("{}", base_string);
        }
        PrintStyle::StartDate => {
            let start_date = anime
                .date_started
                .unwrap_or_else(|| "None".to_string())
                .bright_purple();
            let output_string = format!("{base_string} | {start_date} ");
            println!("{}", output_string);
        }
        PrintStyle::EndDate => {
            let end_date = anime
                .date_completed
                .unwrap_or_else(|| "None".to_string())
                .bright_purple();
            let output_string = format!("{base_string} | {end_date} ");
            println!("{}", output_string);
        }
        PrintStyle::Anki => {
            let cards = anime
                .anki_flashcards
                .unwrap_or(0)
                .to_string()
                .bright_purple();
            let output_string = format!("{base_string} | {cards} ");
            println!("{}", output_string);
        }
        PrintStyle::WatchStatus => {
            let status = anime
                .watch_status
                .unwrap_or(anime_api_data::WatchStatus::Planning)
                .to_string()
                .bright_purple();
            let output_string = format!("{base_string} | {status} ");
            println!("{}", output_string);
        }
    }
}

pub fn list_print(animes: Vec<anime_api_data::Anime>) {
    let anime_iter = animes.iter();

    for anime in anime_iter {
        let name = anime.title.english.bright_yellow();
        let id = anime.id.to_string().bright_red();
        let current_ep = anime
            .episode_progress
            .unwrap_or(0)
            .to_string()
            .bright_green();
        let ep = anime.episodes.to_string().bright_green();
        let slash = "/".to_string().bright_green();

        println!("{}.\t{}\t{}{}{}", id, name, current_ep, slash, ep);
    }
}
