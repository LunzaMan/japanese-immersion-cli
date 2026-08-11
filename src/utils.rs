use crate::{
    anilist_api,
    anime_api_data::{self, Anime, AnimeForExport},
    db,
    error_ctrl::{self, InvalidArgError, invalid_arg_error},
};
use csv::Writer;
use rusqlite::Connection;
use std::{error::Error, path::PathBuf};

pub fn return_anime_if_exists(conn: &Connection, name: Option<&str>) -> Anime {
    let anime: Anime;
    match name {
        Some(name) => {
            if db::anime_by_name_exists(conn, name).unwrap() {
                anime = db::anime_query_by_name(conn, name).unwrap()
            } else {
                invalid_arg_error(InvalidArgError::InvalidName);
            }
        }
        None => {
            if db::current_exists(conn).unwrap() {
                anime = db::query_current(conn).unwrap();
            } else {
                invalid_arg_error(InvalidArgError::Current);
            }
        }
    };

    anime
}

pub fn path_initialization() -> std::path::PathBuf {
    let mut file_path = dirs::data_local_dir().expect("Couldn't load local data directory");

    file_path.push("japanese_immersion_cli/");

    if !file_path.exists() {
        std::fs::create_dir(&file_path).expect("Counldn't create directory");
    }

    file_path.push("anime_db.db3");

    file_path
}

pub fn initialize_export_to_csv(conn: &Connection, path: &Option<PathBuf>) {
    // Check if path exists
    let mut final_path = PathBuf::new();
    match path {
        Some(path) => {
            if path.starts_with("~/") {
                let new_path = path
                    .to_owned()
                    .into_os_string()
                    .into_string()
                    .expect("Couldn't parse into string");

                let x = new_path.trim_start_matches("~/");
                let x_as_path = PathBuf::from(x);

                let home = dirs::home_dir().unwrap();
                final_path.push(home);
                final_path.push(x_as_path);
            } else {
                final_path = path.to_owned();
            }

            if !final_path.exists() {
                error_ctrl::invalid_arg_error(InvalidArgError::InvalidPath);
            }
        }
        None => final_path = dirs::download_dir().unwrap(),
    };

    let _ = export_to_csv(conn, final_path);
}

fn export_to_csv(conn: &Connection, path: PathBuf) -> Result<(), Box<dyn Error>> {
    let animes = db::query_all(conn).unwrap();

    let file = path.join("data.csv");

    let mut wtr = Writer::from_path(file).unwrap();

    for anime in animes {
        wtr.serialize(AnimeForExport {
            id: anime.id,
            episodes: anime.episodes,
            english_title: anime.title.english,
            romaji_title: anime.title.romaji,
            native_title: anime.title.native,
            media_type: anime.media_type,
            url: anime.url,
            anki_flashcards: anime.anki_flashcards,
            is_current: anime.is_current,
            date_started: anime.date_started,
            date_completed: anime.date_completed,
            episode_progress: anime.episode_progress,
            watch_status: anime.watch_status,
            watch_sequence: anime.watch_sequence,
            date_added: anime.date_added,
        })?;
        wtr.flush()?;
    }

    Ok(())
}

pub async fn parse_browse(title: String) -> Vec<anime_api_data::Anime> {
    let result = anilist_api::browse(title).await;
    let filtered_result = result["data"]["Page"]["media"].clone();

    serde_json::from_value::<Vec<anime_api_data::Anime>>(filtered_result)
        .expect("Couldn't create object")
}
