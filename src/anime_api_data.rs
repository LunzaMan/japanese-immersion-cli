use clap::ValueEnum;
use core::fmt;

use rusqlite::types::FromSql;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq, ValueEnum, Clone)]
pub enum WatchStatus {
    Watching,
    Completed,
    Planning,
}

impl fmt::Display for WatchStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            WatchStatus::Watching => write!(f, "Watching"),
            WatchStatus::Planning => write!(f, "Planning"),
            WatchStatus::Completed => write!(f, "Completed"),
        }
    }
}

impl FromSql for WatchStatus {
    fn column_result(value: rusqlite::types::ValueRef<'_>) -> rusqlite::types::FromSqlResult<Self> {
        match value.as_str()? {
            "Watching" => Ok(WatchStatus::Watching),
            "Planning" => Ok(WatchStatus::Planning),
            "Completed" => Ok(WatchStatus::Completed),
            _ => Ok(WatchStatus::Planning),
        }
    }
}

pub enum MediaType {
    Anime,
    // Manga,
    // LightNovel,
}

impl fmt::Display for MediaType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MediaType::Anime => write!(f, "ANIME"),
            // MediaType::Manga => write!(f, "MANGA"),
            // // Never gonna use but if used then i dont know if anilist api uses LN or LIGHTNOVEL
            // MediaType::LightNovel => write!(f, "LN"),
        }
    }
}

#[derive(Clone, ValueEnum)]
pub enum ListType {
    All,
    Watching,
    Planning,
    Completed,
}

impl fmt::Display for ListType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ListType::All => write!(f, "All"),
            ListType::Watching => write!(f, "Watching"),
            ListType::Planning => write!(f, "Planning"),
            ListType::Completed => write!(f, "Completed"),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Anime {
    pub id: u32,
    pub episodes: u16,
    pub title: Title,
    #[serde(rename = "type")]
    pub media_type: String,
    #[serde(rename = "siteUrl")]
    pub url: String,
    pub anki_flashcards: Option<u32>,
    pub is_current: Option<bool>,
    pub date_started: Option<String>,
    pub date_completed: Option<String>,
    pub episode_progress: Option<u16>,
    pub watch_status: Option<WatchStatus>,
    pub watch_sequence: Option<u32>,
    pub date_added: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Title {
    pub english: String,
    pub native: String,
    pub romaji: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AnimeForExport {
    pub id: u32,
    pub episodes: u16,
    pub english_title: String,
    pub romaji_title: String,
    pub native_title: String,
    #[serde(rename = "type")]
    pub media_type: String,
    #[serde(rename = "siteUrl")]
    pub url: String,
    pub anki_flashcards: Option<u32>,
    pub is_current: Option<bool>,
    pub date_started: Option<String>,
    pub date_completed: Option<String>,
    pub episode_progress: Option<u16>,
    pub watch_status: Option<WatchStatus>,
    pub watch_sequence: Option<u32>,
    pub date_added: Option<String>,
}

impl Anime {
    // Getters
    pub fn get_id(&self) -> u32 {
        self.id
    }

    pub fn get_episode_count(&self) -> u16 {
        self.episodes
    }

    pub fn get_media_type(&self) -> &str {
        &self.media_type
    }

    pub fn get_url(&self) -> &str {
        &self.url
    }

    pub fn get_english_title(&self) -> &str {
        &self.title.english
    }

    pub fn get_romaji_title(&self) -> &str {
        &self.title.romaji
    }

    pub fn get_native_title(&self) -> &str {
        &self.title.native
    }
}
