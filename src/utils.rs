use rusqlite::Connection;

use crate::{
    anime_api_data::Anime,
    db,
    error_ctrl::{InvalidArgError, invalid_arg_error},
};

// If name given check if it exists and return. If not then check if current exists then return
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
