pub enum InvalidArgError {
    Current,
    InvalidName,
    Date,
    InvalidEpisodeCount,
}

pub fn invalid_arg_error(err_type: InvalidArgError) -> ! {
    match err_type {
        InvalidArgError::Current => {
            eprint!("No anime is set as current. Set one anime as current before using")
        }
        InvalidArgError::InvalidName => {
            eprint!("Anime not in db. Check if name is correct")
        }
        InvalidArgError::Date => {
            eprint!("Invalid Date Error: Use format YYYY-MM-DD")
        }
        InvalidArgError::InvalidEpisodeCount => {
            eprint!("Error: Invalid episode count")
        }
    };

    std::process::exit(1);
}

pub fn exit_app() -> ! {
    std::process::exit(1)
}
