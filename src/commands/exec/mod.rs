use std::fs;
use std::env;
use std::iter;
use rand::Rng;
use std::path::PathBuf;
use std::io::{ Error, ErrorKind };
use std::time::{ Instant, Duration };

use duct::{ cmd, Expression };
use serenity::model::id::UserId;
use rand::distributions::Alphanumeric;

use lang_manager::LangManager;

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

mod javascript;
pub use self::javascript::JavaScript;

mod csharp;
pub use self::csharp::Csharp;

mod java;
pub use self::java::Java;

mod lua;
pub use self::lua::Lua;

mod ruby;
pub use self::ruby::Ruby;

mod shell;
pub use self::shell::Shell;

mod asmx86;
pub use self::asmx86::Asmx86;

mod asmx64;
pub use self::asmx64::Asmx64;

mod kotlin;
pub use self::kotlin::Kotlin;

mod vb;
pub use self::vb::Vb;

#[derive(Debug, Default)]
pub struct CommandResult {
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    pub timed_out: bool,
}

fn pre_process_code(mut code: String) -> String {
    let re = regex::Regex::new(r"[\u200B-\u200F]").unwrap(); // Invisible characters (Zero-Width Space, Zero Width Non-Joiner, Zero Width Joiner, Left-To-Right Mark, Right-To-Left Mark)
    code = re.replace_all(&code, "").into();

    code
}

fn pre_process_output(mut output: String) -> String {
    output = output.replace("```", "");
    output = output.replace("@everyone", "@ everyone");
    output = output.replace("@here", "@ here");

    output
}

pub type BoxedLang = std::sync::Arc<std::boxed::Box<(dyn language::Language + std::marker::Sync + std::marker::Send + 'static)>>;
pub fn get_lang(lang_manager: &LangManager, lang_code: &str) -> Result<BoxedLang, Error> {
    match lang_manager.get(&lang_code.into()) {
        Some(lang) => {
            if lang_manager.is_language_available(&(*lang)) {
                Ok(lang)
            } else {
                Err(Error::new(ErrorKind::Other, "This programming language is currently unavailable."))
            }
        },
        None => {
            let langs = lang_manager.get_languages_list();
            Err(Error::new(ErrorKind::NotFound, format!("Unknown programming language\nHere are the languages available: {}", langs)))
        }
    }
}

pub fn run_code(cpu_load: Option<&str>, ram_load: Option<&str>, mut code: String, lang: BoxedLang, author: UserId) -> Result<(CommandResult, CommandResult, String, String), Error> {
    let src_path = match save_code(&code, author, &lang.get_source_file_ext()) {
        Ok(path) => path,
        Err(e) => {
            return Err(Error::new(ErrorKind::Other, format!("An error occurred: {}", e)));
        },
    };
    info!("Saving {} code in {}...", lang.get_lang_name(), src_path.to_str().unwrap());
    code = pre_process_code(code);
    if let Some(modified) = lang.pre_process_code(&code, &src_path) {
        match fs::write(src_path.as_path(), &modified) {
            Ok(_) => {},
            Err(e) => {
                return Err(Error::new(ErrorKind::Other, format!("An error occurred: {}", e)));
            },
        };
        code = modified;
    }

    let path_in_container = PathBuf::from("/home").join(src_path.file_name().unwrap());
    let image = lang.get_image_name();
    let out_path = lang.get_out_path(&path_in_container);

    // Start container
    let cmd = cmd!("docker", "run", "--network=none", "--cpus", cpu_load.unwrap_or("0.000"), "--memory", ram_load.unwrap_or("0"), "-t", "-d", image);
    let container_id = cmd.stdout_capture().read()?;
    let cleanup = || {
        let _ = fs::remove_file(&src_path);
        let _ = cmd!("docker", "kill", &container_id).stdout_capture().stderr_capture().run();
        let _ = cmd!("docker", "rm", &container_id).stdout_capture().stderr_capture().run();
    };

    // Copy source file to container
    let cmd = cmd!("docker", "cp", src_path.to_str().unwrap(), format!("{}:{}", container_id, path_in_container.to_str().unwrap()));
    match cmd.run() {
        Ok(_) => { },
        Err(e) => {
            cleanup();
            return Err(Error::new(ErrorKind::Other, format!("Could not copy code snippet to container: {}", e)));
        }
    };

    // Compile code if necessary
    let compilation: Result<CommandResult, Error> = match lang.get_compiler_command(&path_in_container, &out_path) {
        Some(command) => {
            let commands = command.split("&&").map(|command| command.trim());
            let mut res = Ok(CommandResult::default());
            info!("Compiling {} code", lang.get_lang_name());
            for command in commands {
                let mut args = vec!["exec", "-w", "/home", &container_id];
                command.split(' ').for_each(|part| args.push(part));

                let cmd = duct::cmd("docker", args);

                res = match run_command(cmd, 30) {
                    Ok(res) => Ok(res),
                    Err(e) => {
                        cleanup();
                        return Err(Error::new(ErrorKind::Other, format!("An error occurred while compiling code snippet: {}", e)));
                    }
                };
            }
            res
        },
        None => {
            // For interpreted languages, we just copy the source file to the destination path
            let cmd = cmd!("docker", "cp", src_path.to_str().unwrap(), format!("{}:{}", container_id, out_path.to_str().unwrap()));
            let _ = cmd.run();
            Ok(CommandResult::default())
        },
    };
    // Exit prematurely if compilation fails
    let compilation = match compilation {
        Ok(res) => res,
        Err(e) => {
            cleanup();
            return Err(Error::new(ErrorKind::Other, format!("An error occurred while compiling code snippet: {}", e)));
        },
    };

    // Execute code
    let execution = match compilation.exit_code {
        Some(code) if code != 0 => {
            // Return a default value if compilation failed
            CommandResult::default()
        },
        _ => {
            // Compilation succeeded, run the snippet
            info!("Executing {} code", lang.get_lang_name());
            let exec_command = lang.get_execution_command(&out_path);
            let mut args = vec!["exec", "-w", "/home", &container_id];
            exec_command.split(' ').for_each(|part| args.push(part));
            let cmd = duct::cmd("docker", args);
            match run_command(cmd, 10) {
                Ok(res) => res,
                Err(e) => {
                    cleanup();
                    return Err(Error::new(ErrorKind::Other, format!("An error occurred while running code snippet: {}", e)));
                }
            }
        }
    };

    cleanup();
    Ok((compilation, execution, code, lang.get_lang_name()))
}

command!(exec(ctx, msg, _args) {
    let arg = msg.content.clone();
    let split = arg.split("```");
    let data = ctx.data.lock();
    let (command_prefix, cpu_load, ram_load) = {
        let settings = data.get::<::Settings>().unwrap().lock().unwrap();
        (
            settings.command_prefix.clone(),
            settings.cpu_load.clone(),
            settings.ram_load.clone(),
        )
    };

    let langs = data.get::<::LangManager>().unwrap().lock().unwrap().get_languages_list();
    drop(data);

    if split.clone().nth(1).is_none() {
        let _ = msg.reply(&format!("Please add a code section to your message\nExample:\n{}exec\n\\`\\`\\`language\n**code**\n\\`\\`\\`\nHere are the languages available: {}", command_prefix, langs));
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

    let (mut compilation, mut execution, lang) = {
        let lang = {
            // We make sure to lock the data in a separate code block,
            // Otherwise we would block the mutex through the entire compiling and/or executing phases
            let data = ctx.data.lock();
            let mngr = data.get::<::LangManager>().unwrap().lock().unwrap();
            get_lang(&mngr, lang_code.as_ref())
        };
        let lang = match lang {
            Ok(lang) => lang,
            Err(e) => {
                let _ = msg.reply(&format!(":x: {}", e));
                return Ok(());
            }
        };

        {
            let data = ctx.data.lock();
            let db = data.get::<::DbPool>().unwrap();
            match ::models::Snippet::save(code.clone(), &lang.get_lang_name(), msg.author.id, msg.guild_id, db) {
                Ok(_) => {},
                Err(e) => warn!("Could not save snippet to db: {}", e),
            };
        }
        match run_code(Some(&cpu_load), Some(&ram_load), code, lang, msg.author.id) {
            Ok((c, e, _processed_code, l)) => {
                (c, e, l)
            },
            Err(e) => {
                let _ = msg.reply(&e.to_string());
                return Ok(());
            }
        }
    };

    let mut reply = String::new();
    compilation.stderr = pre_process_output(compilation.stderr);
    compilation.stdout = pre_process_output(compilation.stdout);
    execution.stderr = pre_process_output(execution.stderr);
    execution.stdout = pre_process_output(execution.stdout);
    if compilation.timed_out {
        // Compilation timed out
        reply = format!("{}\n:x: Compilation timed out", reply);
    } else if execution.timed_out {
        // Execution timed out
        reply = format!("{}\n:x: Execution timed out", reply);
    } else {
        // Didn't time out
        match compilation.exit_code {
            Some(code) if code != 0 => {
                // Compilation failed
                reply = format!("{}\n:x: Compilation failed: ```\n{}```", reply, compilation.stderr);
            },
            _ => {
                // Compilation succeeded
                if !compilation.stdout.is_empty() {
                    reply = format!("{}\nCompilation output: ```\n{}```", reply, compilation.stdout);
                }
                if !compilation.stderr.is_empty() {
                    reply = format!("{}\nCompilation error output: ```\n{}```", reply, compilation.stderr);
                }
                if let Some(code) = execution.exit_code {
                    reply = format!("{}\nExit code: {}", reply, code);
                }
                if !execution.stdout.is_empty() {
                    reply = format!("{}\nStandard output: ```\n{}```", reply, execution.stdout);
                }
                if !execution.stderr.is_empty() {
                    reply = format!("{}\nError output: ```\n{}```", reply, execution.stderr);
                }
            }
        };
    }

    {
        let data = ctx.data.lock();
        let db = data.get::<::DbPool>().unwrap();
        let mut stat = ::models::LangStat::get(&lang, db);
        stat.increment_snippets_count(db);
    }

    if !reply.is_empty() {
        let header = format!("<@{}>,", msg.author.id);
        let max_msg_len = 2000;
        reply = format!("{}{}", header, reply);
        reply.truncate(max_msg_len - 3);
        if reply.len() == max_msg_len - 3 {
            reply.push_str("```");
        }
        if let Err(e) = msg.channel_id.say(&reply) {
            error!("An error occured while replying to an exec query: {}", e);
            return Ok(());
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

pub fn get_snippets_directory() -> Result<PathBuf, Error> {
    let mut dir = PathBuf::new();
    dir.push(env::current_dir()?);
    dir.push("snippets");
    if !dir.exists() {
        fs::create_dir_all(&dir)?;
    }

    Ok(dir)
}

pub fn get_snippets_directory_for_user(user: UserId) -> Result<PathBuf, Error> {
    let mut dir = get_snippets_directory()?;
    dir.push(user.to_string());
    if !dir.exists() {
        fs::create_dir_all(dir.as_path())?;
    }
    //if ::is_running_as_docker_container() {
    //    cmd!("chown", "dev", &dir).run()?;
    //}

    Ok(dir)
}

fn save_code(code: &str, author: UserId, ext: &str) -> Result<PathBuf, Error> {
    let mut path = get_snippets_directory_for_user(author)?;

    loop {
        path.push(get_random_filename(ext));
        if !path.exists() {
            break;
        }
    }
    fs::write(path.as_path(), code)?;

    //if ::is_running_as_docker_container() {
    //    let _ = cmd!("chown", "dev", path.as_path()).run();
    //}

    Ok(path)
}

fn run_command(cmd: Expression, timeout: u64) -> Result<CommandResult, Error> {
    let child = cmd
        .unchecked() // important! allows us to get stderr instead of an `Error` if the process exits with a non-zero exit code
        .stdout_capture()
        .stderr_capture()
        .start()?;

    let timeout = Duration::from_secs(timeout);
    let start = Instant::now();

    loop {
        match child.try_wait()? {
            Some(_) => {
                break;
            },
            None => {},
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
