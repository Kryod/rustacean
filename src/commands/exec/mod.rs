use std::fs;
use std::env;
use std::iter;
use rand::Rng;
use std::path::PathBuf;
use std::io::{ Error, ErrorKind };
use std::time::{ Instant, Duration };
use rand::distributions::Alphanumeric;
use duct::Expression;

pub mod language;

mod rust;
pub use self::rust::Rust;

mod c;
pub use self::c::C;

mod cpp;
pub use self::cpp::Cpp;

mod php;
pub use self::php::Php;

mod python;
pub use self::python::Python;

mod js;
pub use self::js::JavaScript;

mod csharp;
pub use self::csharp::Csharp;

mod java;
pub use self::java::Java;

mod lua;
pub use self::lua::Lua;

mod shell;
pub use self::shell::Shell;

mod asm;
pub use self::asm::Asm;

mod vb;
pub use self::vb::Vb;

#[derive(Debug, Default)]
pub struct CommandResult {
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    pub timed_out: bool,
}

command!(exec(ctx, msg, _args) {
    let arg = msg.content.clone();
    let split = arg.split("```");
    let data = ctx.data.lock();
    let settings = data.get::<::Settings>().unwrap().clone();
    let langs = data.get::<::LangManager>().unwrap().lock().unwrap().get_languages_list();
    drop(data);

    if split.clone().nth(1).is_none() {
        let _ = msg.reply(&format!("Please add a code section to your message\r\nExample:\r\n{}exec\r\n\\`\\`\\`language\r\n**code**\r\n\\`\\`\\`\nHere are the languages available: {}", settings["command_prefix"], langs));
        return Ok(());
    }
    let code = split
        .take(2)
        .collect::<Vec<_>>()[1];

    let mut split = code.split("\n");
    let (lang_code, mut code) = match split.next() {
        Some(line) => {
            let code = split.collect::<Vec<_>>().join("\n");
            let lang = line.trim().to_ascii_lowercase();
            (lang, code)
        },
        None => {
            let _ = msg.reply(&format!(":x: Please specify a language\nHere are the languages available: {}", langs));
            return Ok(());
        },
    };

    let data = ctx.data.lock();
    let lang = {
        let lang_manager = data.get::<::LangManager>().unwrap().lock().unwrap();
        match lang_manager.get(&lang_code) {
            Some(lang) => {
                if lang_manager.is_language_available(&(*lang)) {
                    lang
                } else {
                    let _ = msg.reply(&format!(":x: This programming language is currently unavailable."));
                    return Ok(());
                }
            },
            None => {
                let _ = msg.reply(&format!(":x: Unknown programming language\nHere are the languages available: {}", langs));
                return Ok(());
            }
        }
    };
    drop(data);

    let src_path = match save_code(&code, &msg.author, &lang.get_source_file_ext()) {
        Ok(path) => path,
        Err(e) => {
            let _ = msg.reply(&format!("Error: {}", e));
            error!("Could not save code snippet: {}", e);
            return Ok(());
        },
    };
    if let Some(modified) = lang.pre_process_code(&code, &src_path) {
        match fs::write(src_path.as_path(), modified) {
            Ok(_) => {},
            Err(e) => error!("Could not save code snippet: {}", e),
        };
    }
    info!("Saved {} code in {}", lang.get_lang_name(), src_path.to_str().unwrap());

    info!("Compiling/Executing {} code", lang.get_lang_name());
    let out_path = lang.get_out_path(&src_path);
    let compilation = match lang.get_compiler_command(&src_path, &out_path) {
        Some(command) => run_command(&src_path, command),
        None => Ok(CommandResult::default())
    };
    let compilation = match compilation {
        Ok(res) => res,
        Err(e) => {
            let err = format!("An error occurred while compiling code snippet: {}", e);
            let _ = msg.reply(&err);
            error!("{}", err);
            return Ok(());
        },
    };

    let execution = match compilation.exit_code {
        Some(code) if code != 0 => {
            // Return a default value if compilation failed
            CommandResult::default()
        },
        _ => {
            // Compilation succeeded, run the snippet
            match run_command(&src_path, lang.get_execution_command(&out_path)) {
                Ok(res) => res,
                Err(e) => {
                    let err = format!("An error occurred while running code snippet: {}", e);
                    let _ = msg.reply(&err);
                    error!("{}", err);
                    return Ok(());
                }
            }
        }
    };

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
                if !compilation.stdout.is_empty() {
                    reply = format!("{}\r\nCompilation output: ```\r\n{}```", reply, compilation.stdout);
                }
                if !compilation.stderr.is_empty() {
                    reply = format!("{}\r\nCompilation error output: ```\r\n{}```", reply, compilation.stderr);
                }
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

    if !reply.is_empty() {
        let header = format!("<@{}>,", msg.author.id);
        reply = format!("{}{}", header, reply);
        reply.truncate(2000);
        if let Err(e) = msg.channel_id.say(&reply) {
            error!("An error occured while replying to an exec query: {}", e);
        }
    } else {
        debug!("Output is empty");
    }

    info!("Done");
});

fn get_random_filename(ext: &str) -> String {
    let mut rng = ::rand::thread_rng();
    let mut name: String;
    loop {
        name = iter::repeat(())
            .map(| _ | rng.sample(Alphanumeric))
            .take(10)
            .collect();
        if name.chars().next().unwrap().is_alphabetic() {
            break;
        }
    }
    name.push_str(ext);

    name
}

pub fn get_snippets_directory() -> PathBuf {
    PathBuf::from("snippets")
}

pub fn save_code(code: &str, author: &serenity::model::user::User, ext: &str) -> Result<PathBuf, Error> {
    let mut path = PathBuf::new();
    let cwd = env::current_dir()?;

    path.push(&cwd);
    path.push(get_snippets_directory());
    path.push(author.id.to_string());
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

pub fn run_command(path: &PathBuf, cmd: Expression) -> Result<CommandResult, Error> {
    let dir = path.parent().unwrap();
    let cmd = cmd.dir(dir).env_remove("RUST_LOG").unchecked();
    let res = run_with_timeout(10, cmd)?;

    Ok(res)
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
