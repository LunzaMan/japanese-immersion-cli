pub enum InvalidArgError {
    Current,
    InvalidName,
    Date,
    InvalidPath,
    InvalidChoice,
    InvalidAnime,
    Completed,
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
        InvalidArgError::InvalidPath => {
            eprint!("Error: Path doesn't exist")
        }
        InvalidArgError::InvalidChoice => {
            eprint!("Error: Invalid Choice")
        }
        InvalidArgError::InvalidAnime => {
            eprint!("Error: No such anime found")
        }
        InvalidArgError::Completed => {
            eprint!("Error: Anime already Completed, set to watching to update")
        }
    };

    std::process::exit(1);
}

pub fn exit_app() -> ! {
    std::process::exit(1)
}
