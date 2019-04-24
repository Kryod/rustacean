use std::process::Command;

fn update(migration_dir: &str, url: &str) {
    println!("Running database update...");

    let out = Command::new("diesel")
        .args(&[ "migration", "run", "--migration-dir", migration_dir, "--database-url", url ])
        .output()
        .expect("Could not run the database update.");

    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    print!("{}\r\n{}", stdout, stderr);
}

/// Runs database update scripts if necessary.
/// 
/// Use it with `cargo run update-db`.
/// 
/// This command should be used after pulling from the repository.
pub fn update_db() {
    update("./migrations", "rustacean.sqlite3");
}
