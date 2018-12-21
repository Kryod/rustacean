use std::env;

pub mod update_db;

pub fn tools() -> bool {
    let mut args = env::args();
    args.next().unwrap(); // Discard the program's path
    let mut command = match args.next() {
        Some(c) => c,
        None => {
            return false;
        }
    };
    command = command.trim().into();
    command = command.to_lowercase().into();

    let command = command.as_str();

    match command {
        "update-db" => update_db::update_db(),
        _ => return false,
    };

    true
}
