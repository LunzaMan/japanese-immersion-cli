use chrono::NaiveDate;
use rusqlite::{Connection, Params, Result, Statement, named_params, params, params_from_iter};

use crate::{
    CardsCommand, DateType, EpisodeMutation,
    anime_api_data::{
        self, Anime, ListType,
        MediaType::{self},
        WatchStatus,
    },
    error_ctrl::{InvalidArgError, invalid_arg_error},
};
use std::path::PathBuf;

pub fn connect(db_path: PathBuf) -> Connection {
    let conn = Connection::open(db_path).expect("Can't connect to db");

    if !conn.table_exists(None, "anime").unwrap() {
        conn.execute(
            "
                CREATE TABLE IF NOT EXISTS anime(
                    id INTEGER PRIMARY KEY,
                    english_name TEXT,
                    romaji_name TEXT,
                    native_name TEXT,
                    date_started TEXT,
                    date_completed TEXT, 
                    date_added TEXT,
                    total_episodes INTEGER, 
                    episode_progress INTEGER, 
                    anki_flashcards INTEGER, 
                    is_current INTEGER,
                    anilist_url TEXT,
                    watch_sequence INTEGER,
                    watch_status TEXT,
                    media_type TEXT
                ), STRICT
            ",
            (),
        )
        .expect("Table creation failed");

        conn.execute(
            "CREATE UNIQUE INDEX one_current ON anime(is_current) WHERE is_current=1",
            [],
        )
        .expect("Failed creating unique index");
    }

    conn
}

pub fn add_anime(
    conn: &Connection,
    api_data: &anime_api_data::Anime,
    watch_status: anime_api_data::WatchStatus,
    is_current: bool,
) -> Result<()> {
    println!("Adding Anime to Database");

    println!("{}", is_current);

    if is_current {
        conn.execute(
            "UPDATE anime SET is_current = false WHERE is_current = true",
            (),
        )
        .expect("Couldn't remove is_current from previous anime");
    }

    // todo: first check if anime already in list by using anilist id
    conn.execute(
            "INSERT INTO anime(id, english_name, romaji_name, native_name, date_added, total_episodes, is_current, anilist_url, watch_status, media_type) 
                 VALUES(?1, ?2, ?3, ?4, current_date, ?5, ?6, ?7, ?8, ?9 )",
                 (
                     api_data.get_id(),
                     api_data.get_english_title(),
                     api_data.get_romaji_title(),
                     api_data.get_native_title(),
                     api_data.get_episode_count(),
                     is_current,
                     api_data.get_url(),
                     watch_status.to_string(),
                     api_data.get_media_type()
                     ),
        ).expect("Failed to add anime");

    if is_current {
        let id: u32 = api_data.get_id();
        conn.execute(
            "UPDATE anime SET date_started = current_date WHERE id=?1",
            params![id],
        )
        .expect("Couldn't add date started");
    }

    println!("Anime added to database");

    Ok(())
}

pub fn query_current(conn: &Connection) -> rusqlite::Result<Anime> {
    let mut stmt = conn.prepare("select * from anime where is_current=1")?;

    let mut result = anime_query(&mut stmt, []).unwrap();

    let anime = result.remove(0);
    Ok(anime)
}

pub fn current_exists(conn: &Connection) -> rusqlite::Result<bool> {
    let sql = "select exists( select 1 from anime where is_current=1)";

    let result: bool = conn.query_row(sql, [], |row| row.get(0))?;

    Ok(result)
}

pub fn query_list(
    conn: &Connection,
    list_type: ListType,
    media_type: MediaType,
) -> rusqlite::Result<Vec<anime_api_data::Anime>> {
    let mut stmt;
    let mut parameters = Vec::new();
    match media_type {
        MediaType::Anime => match list_type {
            ListType::All => {
                stmt = conn.prepare("SELECT * FROM anime")?;
            }
            _ => {
                stmt = conn.prepare("SELECT * FROM anime WHERE watch_status = :list_type")?;
                parameters.push(list_type.to_string().trim().to_owned());
            }
        },
        // _ => {
        //     stmt = conn.prepare("SELECT * FROM anime")?;
        // }
    }

    let result = anime_query(&mut stmt, params_from_iter(parameters)).unwrap();
    Ok(result)
}

pub fn anime_query_by_name(conn: &Connection, name: &str) -> rusqlite::Result<Anime> {
    let sql = format!(
        "
        select * from anime
        where english_name = '{name}' COLLATE NOCASE
        OR romaji_name = '{name}' COLLATE NOCASE
        OR native_name = '{name}' COLLATE NOCASE;
        "
    );

    let mut stmt = conn.prepare(&sql)?;
    let result = anime_query(&mut stmt, []).unwrap();
    let anime = result.into_iter().next().unwrap();

    Ok(anime)
}

pub fn anime_by_name_exists(conn: &Connection, name: &str) -> rusqlite::Result<bool> {
    let sql = "
        SELECT EXISTS( SELECT 1 FROM anime 
        WHERE english_name = :name COLLATE NOCASE
        OR romaji_name = :name COLLATE NOCASE
        OR native_name = :name COLLATE NOCASE);
    ";

    let result: bool = conn.query_row(&sql, named_params! {":name": name}, |row| row.get(0))?;

    Ok(result)
}

fn anime_query<P: Params>(
    stmt: &mut Statement<'_>,
    params: P,
) -> rusqlite::Result<Vec<anime_api_data::Anime>> {
    let result: Vec<anime_api_data::Anime> = stmt
        .query_map(params, |row| {
            Ok(anime_api_data::Anime {
                id: row.get(0)?,
                title: anime_api_data::Title {
                    english: row.get(1)?,
                    romaji: row.get(2)?,
                    native: row.get(3)?,
                },
                date_started: row.get(4)?,
                date_completed: row.get(5)?,
                date_added: row.get("date_added")?,
                episodes: row.get("total_episodes")?,
                episode_progress: row.get("episode_progress")?,
                anki_flashcards: row.get("anki_flashcards")?,
                is_current: row.get("is_current")?,
                url: row.get("anilist_url")?,
                watch_sequence: row.get("watch_sequence")?,
                watch_status: row.get("watch_status")?,
                media_type: row.get("media_type")?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(result)
}

pub fn add_card_mutation(
    conn: &Connection,
    add_type: &CardsCommand,
    number_of_cards: &u32,
    name: &Option<&str>,
) -> Result<()> {
    match (add_type, name) {
        (CardsCommand::Add, None) => conn.execute(
            "UPDATE anime SET anki_flashcards=COALESCE(anki_flashcards,0) + ?1 WHERE is_current=1",
            [number_of_cards],
        )?,
        (CardsCommand::Add, Some(name)) => conn.execute(
            "UPDATE anime 
            SET anki_flashcards= COALESCE(anki_flashcards,0) + :number
            where english_name = :name COLLATE NOCASE
            OR romaji_name = :name COLLATE NOCASE
            OR native_name = :name COLLATE NOCASE;
            ",
            named_params! {
                ":number" : number_of_cards,
                ":name" : name,
            },
        )?,
        (CardsCommand::Total, Some(name)) => conn.execute(
            "UPDATE anime 
            SET anki_flashcards= :number
            where english_name = :name COLLATE NOCASE
            OR romaji_name = :name COLLATE NOCASE
            OR native_name = :name COLLATE NOCASE;
            ",
            named_params! {
                ":number" : number_of_cards,
                ":name" : name,
            },
        )?,

        (CardsCommand::Total, None) => conn.execute(
            "UPDATE anime SET anki_flashcards=?1 WHERE is_current=1",
            [number_of_cards],
        )?,
    };

    Ok(())
}

pub fn episode_mutation(
    conn: &Connection,
    episode_mutation_type: &EpisodeMutation,
    name: &Option<&str>,
    number: Option<u16>,
) -> Result<()> {
    let base = "UPDATE anime";
    let where_clause = match name {
        Some(name) => {
            let name_ = name;
            format!(
                "WHERE english_name = '{name_}' COLLATE NOCASE
            OR romaji_name = '{name_}' COLLATE NOCASE
            OR native_name = '{name_}' COLLATE NOCASE
           ; "
            )
        }
        None => format!("WHERE is_current=1;"),
    };

    let set_clause: String;

    match (episode_mutation_type, number) {
        (_, Some(number)) => {
            set_clause = format!("SET episode_progress = {number}");
        }
        (EpisodeMutation::Add, _) => {
            set_clause = format!("SET episode_progress = COALESCE(episode_progress,0) + 1");
        }
        (EpisodeMutation::Subtract, _) => {
            set_clause = format!("SET episode_progress = COALESCE(episode_progress,0) - 1");
        }
        (_, _) => {
            set_clause = format!("");
        }
    };

    let sql = format!("{base}\n{set_clause}\n{where_clause}");

    conn.execute(&sql, [])?;

    Ok(())
}

pub fn remove_current(conn: &Connection) -> rusqlite::Result<()> {
    let remove_current_sql = "UPDATE anime SET is_current = 0 WHERE is_current = 1;";
    conn.execute(remove_current_sql, [])?;
    Ok(())
}

pub fn set_current(conn: &Connection, name: &str) -> rusqlite::Result<()> {
    let watching_status = WatchStatus::Watching.to_string();
    let set_current_sql = format!(
        "
        UPDATE anime SET is_current = 1,
        watch_status = '{watching_status}'
        WHERE english_name = '{name}' COLLATE NOCASE
        OR romaji_name = '{name}' COLLATE NOCASE 
        OR native_name = '{name}' COLLATE NOCASE;
        "
    );

    conn.execute(&set_current_sql, [])?;

    Ok(())
}

fn is_valid_date(date: &str) -> bool {
    let is_valid = NaiveDate::parse_from_str(date, "%Y-%m-%d").is_ok();

    is_valid
}

pub fn date_mutation(
    conn: &Connection,
    name: &str,
    date: Option<&str>,
    date_type: &DateType,
) -> rusqlite::Result<()> {
    let sql_head = "UPDATE anime";
    let where_clause = "
            WHERE english_name = ?1 COLLATE NOCASE
            OR romaji_name = ?1 COLLATE NOCASE
            OR native_name = ?1 COLLATE NOCASE;
        ";
    let mut params = Vec::new();
    params.push(name);
    let set_clause: &str;

    match (date_type, date) {
        (DateType::Start, Some(date)) => {
            let is_valid_date = is_valid_date(date);
            if is_valid_date {
                params.push(date);
                set_clause = "SET date_started = ?2";
            } else {
                invalid_arg_error(InvalidArgError::Date);
            }
        }
        (DateType::Start, None) => {
            set_clause = "SET date_started = current_date";
        }
        (DateType::End, Some(date)) => {
            let is_valid_date = is_valid_date(date);
            if is_valid_date {
                params.push(date);
                set_clause = "SET date_completed = ?2"
            } else {
                invalid_arg_error(InvalidArgError::Date);
            }
        }
        (DateType::End, None) => set_clause = "SET date_completed = current_date",
    };

    let sql = format!("{sql_head}\n{set_clause}\n{where_clause}");

    conn.execute(&sql, params_from_iter(params))?;

    Ok(())
}

pub fn watch_status_mutation(
    conn: &Connection,
    watch_status: &anime_api_data::WatchStatus,
    name: Option<&str>,
) -> rusqlite::Result<()> {
    let sql_header = "UPDATE anime SET watch_status = ?1";

    let sql_footer;
    let mut params = Vec::new();
    params.push(watch_status.to_string());
    match name {
        Some(name) => {
            params.push(name.to_string());
            sql_footer = "
            WHERE english_name = ?2 COLLATE NOCASE
            OR romaji_name = ?2 COLLATE NOCASE
            OR native_name = ?2 COLLATE NOCASE;
                ";
        }
        None => sql_footer = "WHERE is_current = 1",
    };

    let sql = format!("{sql_header}\n{sql_footer}");
    conn.execute(&sql, params_from_iter(params))?;
    Ok(())
}

pub fn query_all(conn: &Connection) -> rusqlite::Result<Vec<Anime>> {
    let mut sql = conn.prepare(
        "
        SELECT * from anime;
        ",
    )?;

    let anime = anime_query(&mut sql, [])?;

    Ok(anime)
}
