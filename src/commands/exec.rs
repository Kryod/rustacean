use std::fs;
use std::env;
use std::iter;
use rand::Rng;
use std::path::PathBuf;
use std::process::Command;
use std::io::{ Error, ErrorKind };
use rand::distributions::Alphanumeric;


command!(exec(_ctx, msg, _args) {
    let arg = msg.content.clone();
    let mut code = arg.split("```")
            .take(2)
            .collect::<Vec<_>>()[1];

    if code.get(0..2) != Some("rs") {
        let _ = msg.channel_id.say("PLIZ CODE IN RUST!");
    } else {

        code = code.get(2 .. code.len()).unwrap();

        let path = match save_code(code, "some_dir") {
            Ok(path) => {
                match run_code(path) {
                    Ok((stdout, stderr)) => {
                        let _ = msg.channel_id.say(format!("stdout: {}", stdout));
                        let _ = msg.channel_id.say(format!("stderr: {}", stderr));
                    },
                    Err(e) => {
                        eprintln!("An error occurred while running code snippet: {}", e);
                        let _ = msg.channel_id.say(format!("error: {}", e));
                    },
                };
            },
            Err(e) => {
                eprintln!("Could not save code snippet: {}", e);
            },
        };
    }
});

fn get_random_filename() -> String {
    let mut rng = ::rand::thread_rng();
    let mut name: String = iter::repeat(())
        .map(| _ | rng.sample(Alphanumeric))
        .take(10)
        .collect();
    name.push_str(".rs");

    name
}

fn save_code(code: &str, dir_name: &str) -> Result<PathBuf, Error> {
    let mut path = PathBuf::new();
    let cwd = env::current_dir()?;

    path.push(&cwd);
    path.push(dir_name);
    fs::create_dir_all(path.as_path())?;

    loop {
        path.push(get_random_filename());
        if !path.exists() {
            break;
        }
    }
    fs::write(path.as_path(), code)?;

    Ok(path)
}

fn run_code(file_path: PathBuf) -> Result<(String, String), Error> {
    let copy = file_path.clone();
    let dir = copy.parent().unwrap();

    let file_name = file_path.file_name().unwrap();
    let exe_name = format!("{}.out", file_path.to_str().unwrap());

    Command::new("rustc")
        .current_dir(dir)
        .arg(file_name)
        .arg("-o")
        .arg(&exe_name)
        .output()
        .expect("Failed to execute rustc");

    let output = Command::new(exe_name)
        .current_dir(dir)
        .output()?;
    let stdout = ::std::str::from_utf8(&output.stdout)
        .map_err(| e | Error::new(ErrorKind::InvalidData, e))
        .map(| s | String::from(s))?;
    let stderr = ::std::str::from_utf8(&output.stderr)
        .map_err(| e | Error::new(ErrorKind::InvalidData, e))
        .map(| s | String::from(s))?;

    Ok((stdout, stderr))
}