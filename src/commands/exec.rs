use std::fs;
use std::env;
use std::iter;
use rand::Rng;
use duct::cmd;
use std::path::PathBuf;
use std::collections::HashMap;
use std::io::{ Error, ErrorKind };
use std::time::{ Instant, Duration };
use rand::distributions::Alphanumeric;

command!(exec(_ctx, msg, _args) {
    let arg = msg.content.clone();
    let code = arg.split("```")
        .take(2)
        .collect::<Vec<_>>()[1];

    let mut split = code.split("\n");
    let (language, mut code) = match split.next() {
        Some(line) => {
            let code = split.collect::<Vec<_>>().join("\n");
            let lang = line.trim().to_ascii_lowercase();
            (lang, code)
        },
        None => {
            let _ = msg.reply(":x: Please specify a language");
            return Ok(());
        },
    };

    let (path, compiler) = match language.as_ref() {
        "c" => {
            info!("Saving C code");
            if let Some(modified) = modif_code_c(&code) {
                code = modified;
            }
            let path = match save_code(&code, "some_dir", ".c") {
                Ok(path) => path,
                Err(e) => {
                    let _ = msg.channel_id.say(format!("Error: {}", e));
                    eprintln!("Could not save code snippet: {}", e);
                    return Ok(());
                },
            };
            (path, "gcc")
        },
        "rs" | "rust" => {
            info!("Saving Rust code");
            if let Some(modified) = modif_code_rust(&code) {
                code = modified;
            }
            let path = match save_code(&code, "some_dir", ".rs") {
                Ok(path) => path,
                Err(e) => {
                    let _ = msg.channel_id.say(format!("Error: {}", e));
                    eprintln!("Could not save code snippet: {}", e);
                    return Ok(());
                },
            };
            (path, "rustc")
        },
        _ => {
            let _ = msg.channel_id.say(":x: Unknown programming language");
            return Ok(());
        }
    };

    info!("Compiling/Executing code");
    match run_code(path, compiler) {
        Ok((compilation, execution)) => {
            let mut reply = String::new();
            if compilation.timed_out {
                // Compilation timed out
                reply = format!("{}\r\n:x: Compilation timed out", reply);
            } else if execution.timed_out {
                // Execution timed out
                reply = format!("{}\r\n:x: Execution timed out", reply);
            } else {
                // Didn't time out
                match compilation.exit_code {
                    Some(code) if code != 0 => {
                        // Compilation failed
                        reply = format!("{}\r\n:x: Compilation failed: ```\r\n{}```", reply, compilation.stderr);
                    },
                    _ => {
                        // Compilation succeeded
                        if let Some(code) = execution.exit_code {
                            reply = format!("{}\r\nExit code: {}", reply, code);
                        }
                        if !execution.stdout.is_empty() {
                            reply = format!("{}\r\nStandard output: ```\r\n{}```", reply, execution.stdout);
                        }
                        if !execution.stderr.is_empty() {
                            reply = format!("{}\r\nError output: ```\r\n{}```", reply, execution.stderr);
                        }
                    }
                };
            }
            info!("Checking Output");
            if !reply.is_empty() {
                let header = format!("<@{}>,", msg.author.id);
                reply = format!("{}{}", header, reply);
                reply.truncate(2000);
                if let Err(e) = msg.channel_id.say(&reply) {
                    eprintln!("An error occured while replying to an exec query: {}", e);
                }
            } else {
                debug!("Output is empty");
            }
        },
        Err(e) => {
            let _ = msg.channel_id.say(format!("Error: {}", e));
            eprintln!("An error occurred while running code snippet: {}", e);
        },
    };

    info!("Done");
});

fn modif_code_c(code: &str) -> Option<String> {
    use regex::Regex;

    let re = Regex::new(r"int\s*main\s*\(.*\)").unwrap();
    if !re.is_match(&code) {
        let result = format!("int main() {{\r\n{}\r\n}}", code);
        return Some(result);
    }

    None
}

fn modif_code_rust(code: &str) -> Option<String> {
    use regex::Regex;

    let re = Regex::new(r"fn\s*main\s*\(\s*\)").unwrap();
    if !re.is_match(&code) {
        let result = format!("fn main() {{\r\n{}\r\n}}", code);
        return Some(result);
    }

    None
}

fn get_random_filename(ext: &str) -> String {
    let mut rng = ::rand::thread_rng();
    let mut name: String = iter::repeat(())
        .map(| _ | rng.sample(Alphanumeric))
        .take(10)
        .collect();
    name.push_str(ext);

    name
}

fn save_code(code: &str, dir_name: &str, ext: &str) -> Result<PathBuf, Error> {
    let mut path = PathBuf::new();
    let cwd = env::current_dir()?;

    path.push(&cwd);
    path.push(dir_name);
    fs::create_dir_all(path.as_path())?;

    loop {
        path.push(get_random_filename(ext));
        if !path.exists() {
            break;
        }
    }
    fs::write(path.as_path(), code)?;

    Ok(path)
}

#[derive(Debug, Default)]
struct CommandResult {
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    pub timed_out: bool,
}

fn run_code(file_path: PathBuf, compiler: &str) -> Result<(CommandResult, CommandResult), Error> {
    let dir = file_path.parent().unwrap();

    let file_name = file_path.to_str().unwrap();
    let exe_name = format!("{}.out", file_name);

    let compilation = run_with_timeout(10, cmd!(compiler, file_name, "-o", &exe_name).dir(dir).env_remove("RUST_LOG").unchecked())?;
    match compilation.exit_code {
        Some(code) if code != 0 => {
            // Short-circuit if something went wrong with the compilation
            return Ok((compilation, CommandResult::default()));
        },
        _ => {},
    };
    let execution = run_with_timeout(10, cmd!(&exe_name).dir(dir).env_remove("RUST_LOG").unchecked())?;
    Ok((compilation, execution))
}

fn run_with_timeout(timeout: u64, cmd: ::duct::Expression) -> Result<CommandResult, Error> {
    let child = cmd
        .stdout_capture()
        .stderr_capture()
        .start()?;

    let timeout = Duration::from_secs(timeout);
    let start = Instant::now();

    loop {
        match child.try_wait() {
            Ok(Some(_)) => {
                break;
            },
            Ok(None) => {},
            Err(e) => return Err(e),
        };

        if start.elapsed() >= timeout {
            child.kill()?;

            return Ok(CommandResult {
                exit_code: None,
                stdout: "".into(),
                stderr: "".into(),
                timed_out: true,
            });
        }

        ::std::thread::sleep(Duration::from_millis(250));
    }

    let output = child.wait()?;

    let stdout = ::std::str::from_utf8(&output.stdout)
        .map_err(| e | Error::new(ErrorKind::InvalidData, e))?
        .to_owned();
    let stderr = ::std::str::from_utf8(&output.stderr)
        .map_err(| e | Error::new(ErrorKind::InvalidData, e))?
        .to_owned();

    Ok(CommandResult {
        exit_code: output.status.code(),
        stdout,
        stderr,
        timed_out: false,
    })
}
