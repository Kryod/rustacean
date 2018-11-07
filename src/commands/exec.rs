use std::fs;
use std::env;
use std::iter;
use rand::Rng;
use duct::cmd;
use std::path::PathBuf;
use std::io::{ Error, ErrorKind };
use std::time::{ Instant, Duration };
use rand::distributions::Alphanumeric;

command!(exec(_ctx, msg, _args) {
    let arg = msg.content.clone();
    let mut code = arg.split("```")
            .take(2)
            .collect::<Vec<_>>()[1];

    if code.get(0..1) == Some("C") || code.get(0..1) == Some("c") {
        code = code.get(1 .. code.len()).unwrap();

        let _ = msg.channel_id.say("C code!");
        match save_code(code, "some_dir", ".c") {
            Ok(path) => {
                match run_code(path, "gcc".to_owned()) {
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
                                    reply = format!("{}\r\n:x: Compilation failed: ``` {} ```", reply, compilation.stderr);
                                },
                                _ => {
                                    // Compilation succeeded
                                    if let Some(code) = execution.exit_code {
                                        reply = format!("{}\r\nExit code: {}", reply, code);
                                    }
                                    if !execution.stdout.is_empty() {
                                        reply = format!("{}\r\nStandard output: ``` {} ```", reply, execution.stdout);
                                    }
                                    if !execution.stderr.is_empty() {
                                        reply = format!("{}\r\nError output: ``` {} ```", reply, execution.stderr);
                                    }
                                }
                            };
                        }
                        if !reply.is_empty() {
                            let header = format!("<@{}>,", msg.author.id);
                            reply = format!("{}{}", header, reply);
                            reply.truncate(2000);
                            if let Err(e) = msg.channel_id.say(&reply) {
                                eprintln!("An error occured while replying to an exec query: {}", e);
                            }
                        }
                    },
                    Err(e) => {
                        let _ = msg.channel_id.say(format!("Error: {}", e));
                        eprintln!("An error occurred while running code snippet: {}", e);
                    },
                };
            },
            Err(e) => {
                let _ = msg.channel_id.say(format!("Error: {}", e));
                eprintln!("Could not save code snippet: {}", e);
            },
        };
    } else if code.get(0..2) == Some("rs") ||  code.get(0..2) == Some("rust") {
        code = code.get(2 .. code.len()).unwrap();

        match save_code(code, "some_dir", ".rs") {
            Ok(path) => {
                match run_code(path, "rustc".to_owned()) {
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
                                    reply = format!("{}\r\n:x: Compilation failed: ``` {} ```", reply, compilation.stderr);
                                },
                                _ => {
                                    // Compilation succeeded
                                    if let Some(code) = execution.exit_code {
                                        reply = format!("{}\r\nExit code: {}", reply, code);
                                    }
                                    if !execution.stdout.is_empty() {
                                        reply = format!("{}\r\nStandard output: ``` {} ```", reply, execution.stdout);
                                    }
                                    if !execution.stderr.is_empty() {
                                        reply = format!("{}\r\nError output: ``` {} ```", reply, execution.stderr);
                                    }
                                }
                            };
                        }
                        if !reply.is_empty() {
                            let header = format!("<@{}>,", msg.author.id);
                            reply = format!("{}{}", header, reply);
                            reply.truncate(2000);
                            if let Err(e) = msg.channel_id.say(&reply) {
                                eprintln!("An error occured while replying to an exec query: {}", e);
                            }
                        }
                    },
                    Err(e) => {
                        let _ = msg.channel_id.say(format!("Error: {}", e));
                        eprintln!("An error occurred while running code snippet: {}", e);
                    },
                };
            },
            Err(e) => {
                let _ = msg.channel_id.say(format!("Error: {}", e));
                eprintln!("Could not save code snippet: {}", e);
            },
        };
    } else {
        let _ = msg.channel_id.say("PLIZ CODE IN RUST!");
    }
});

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

fn run_code(file_path: PathBuf, compilo: String) -> Result<(CommandResult, CommandResult), Error> {
    let dir = file_path.parent().unwrap();

    let file_name = file_path.to_str().unwrap();
    let exe_name = format!("{}.out", file_name);

    let compilation = run_with_timeout(10, cmd!(compilo, file_name, "-o", &exe_name).dir(dir).unchecked())?;
    match compilation.exit_code {
        Some(code) if code != 0 => {
            // Short-circuit if something went wrong with the compilation
            return Ok((compilation, CommandResult::default()));
        },
        _ => {},
    };
    let execution = run_with_timeout(10, cmd!(&exe_name).dir(dir).unchecked())?;
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
