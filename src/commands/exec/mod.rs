use rand::Rng;
use std::env;
use std::fs;
use std::io::{Error, ErrorKind};
use std::iter;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use duct::{cmd, Expression};
use rand::distributions::Alphanumeric;
use serenity::{
    framework::standard::{macros::command, CommandResult},
    model::{channel::Message, id::UserId},
    prelude::Context,
};

use crate::{models, DbPool, LangManager, Settings};

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

mod haskell;
pub use self::haskell::Haskell;

mod kotlin;
pub use self::kotlin::Kotlin;

mod julia;
pub use self::julia::Julia;

mod go;
pub use self::go::Go;

mod typescript;
pub use self::typescript::Typescript;

mod vb;
pub use self::vb::Vb;

mod ocaml;
pub use self::ocaml::OCaml;

mod prolog;
pub use self::prolog::Prolog;

mod pony;
pub use self::pony::Pony;

#[derive(Debug, Default)]
pub struct ExecResult {
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    pub timed_out: bool,
    pub duration: Duration,
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

pub type BoxedLang = std::sync::Arc<
    std::boxed::Box<(dyn language::Language + std::marker::Sync + std::marker::Send + 'static)>,
>;
pub fn get_lang(lang_manager: &LangManager, lang_code: &str) -> Result<BoxedLang, Error> {
    match lang_manager.get(&lang_code.to_string()) {
        Some(lang) => {
            if lang_manager.is_language_available(&(*lang)) {
                Ok(lang)
            } else {
                Err(Error::new(
                    ErrorKind::Other,
                    "This programming language is currently unavailable.",
                ))
            }
        }
        None => {
            let langs = lang_manager.get_languages_list();
            Err(Error::new(
                ErrorKind::NotFound,
                format!(
                    "Unknown programming language\nHere are the languages available: {}",
                    langs
                ),
            ))
        }
    }
}

fn append_to_msg(ctx: &Option<&mut Context>, msg: &mut Option<&mut Message>, line: &str) {
    if let Some(ref mut msg) = msg {
        if let Some(ctx) = ctx {
            let new_content = format!("{}\n{}", msg.content, line);
            let _ = msg.edit(ctx, |m| m.content(new_content));
        };
    };
}

pub fn run_code(
    settings: &Settings,
    mut code: String,
    lang: BoxedLang,
    author: UserId,
    ctx: Option<&mut Context>,
    mut reply: Option<&mut Message>,
) -> Result<(ExecResult, ExecResult, String, String), Error> {
    append_to_msg(&ctx, &mut reply, "Saving code...");
    let src_path = match save_code(&code, author, &lang.get_source_file_ext()) {
        Ok(path) => path,
        Err(e) => {
            return Err(Error::new(
                ErrorKind::Other,
                format!("An error occurred: {}", e),
            ));
        }
    };
    info!(
        "Saved {} code in {}.",
        lang.get_lang_name(),
        src_path.to_str().unwrap()
    );

    code = pre_process_code(code);
    if let Some(modified) = lang.pre_process_code(&code, &src_path) {
        match fs::write(src_path.as_path(), &modified) {
            Ok(_) => {}
            Err(e) => {
                return Err(Error::new(
                    ErrorKind::Other,
                    format!("An error occurred: {}", e),
                ));
            }
        };
        code = modified;
    }

    let path_in_container = PathBuf::from("/home").join(src_path.file_name().unwrap());
    let image = lang.get_image_name();
    let out_path = lang.get_out_path(&path_in_container);

    // Start container
    append_to_msg(&ctx, &mut reply, "Starting session...");
    let cmd = cmd!(
        "docker",
        "run",
        "--network=none",
        "--kernel-memory",
        &settings.kernel_memory,
        "--cpus",
        &settings.cpu_load,
        "--memory",
        &settings.ram_load,
        "-t",
        "-d",
        image
    );
    let container_id = cmd.stdout_capture().read()?;
    let cleanup = || {
        let _ = fs::remove_file(&src_path);
        let _ = cmd!("docker", "kill", &container_id)
            .stdout_capture()
            .stderr_capture()
            .run();
        let _ = cmd!("docker", "rm", &container_id)
            .stdout_capture()
            .stderr_capture()
            .run();
    };

    // Copy source file to container
    append_to_msg(&ctx, &mut reply, "Copying code snippet...");
    let cmd = cmd!(
        "docker",
        "cp",
        src_path.to_str().unwrap(),
        format!("{}:{}", container_id, path_in_container.to_str().unwrap())
    );
    match cmd.run() {
        Ok(_) => {}
        Err(e) => {
            cleanup();
            return Err(Error::new(
                ErrorKind::Other,
                format!("Could not copy code snippet to container: {}", e),
            ));
        }
    };

    // Compile code if necessary
    let compilation: Result<ExecResult, Error> =
        match lang.get_compiler_command(&path_in_container, &out_path) {
            Some(command) => {
                append_to_msg(&ctx, &mut reply, "Compiling code snippet...");
                let commands = command.split("&&").map(|command| command.trim());
                let mut res = Ok(ExecResult::default());
                info!("Compiling {} code", lang.get_lang_name());
                for command in commands {
                    let mut args = vec!["exec", "-w", "/home", &container_id];
                    command.split(' ').for_each(|part| args.push(part));

                    let cmd = duct::cmd("docker", args);

                    res = match run_command(cmd, settings.compilation_timeout) {
                        Ok(res) => Ok(res),
                        Err(e) => {
                            cleanup();
                            return Err(Error::new(
                                ErrorKind::Other,
                                format!("An error occurred while compiling code snippet: {}", e),
                            ));
                        }
                    };
                }
                res
            }
            None => {
                // For interpreted languages, we just copy the source file to the destination path
                let cmd = cmd!(
                    "docker",
                    "cp",
                    src_path.to_str().unwrap(),
                    format!("{}:{}", container_id, out_path.to_str().unwrap())
                );
                let _ = cmd.run();
                Ok(ExecResult::default())
            }
        };
    // Exit prematurely if compilation fails
    let compilation = match compilation {
        Ok(res) => res,
        Err(e) => {
            cleanup();
            return Err(Error::new(
                ErrorKind::Other,
                format!("An error occurred while compiling code snippet: {}", e),
            ));
        }
    };

    // Execute code
    let execution = if compilation.timed_out {
        ExecResult::default()
    } else {
        match compilation.exit_code {
            Some(code) if code != 0 => {
                // Return a default value if compilation failed
                ExecResult::default()
            }
            _ => {
                // Compilation succeeded, run the snippet
                append_to_msg(&ctx, &mut reply, "Running code snippet...");
                info!("Executing {} code", lang.get_lang_name());
                let exec_command = lang.get_execution_command(&out_path);
                let mut args = vec!["exec", "-w", "/home", &container_id];
                exec_command.split(' ').for_each(|part| args.push(part));
                let cmd = duct::cmd("docker", args);
                match run_command(cmd, settings.execution_timeout) {
                    Ok(res) => res,
                    Err(e) => {
                        cleanup();
                        return Err(Error::new(
                            ErrorKind::Other,
                            format!("An error occurred while running code snippet: {}", e),
                        ));
                    }
                }
            }
        }
    };

    append_to_msg(&ctx, &mut reply, "Closing session...");
    cleanup();
    Ok((compilation, execution, code, lang.get_lang_name()))
}

#[command]
#[aliases("execute", "run", "code")]
#[description = "Executes a code snippet. Your message needs to look like this:\r\n~exec\r\n\\`\\`\\`language\r\n\r\ncode...\r\n\\`\\`\\`\r\nwhere `language` is the language of your choice.\r\nFor example:\r\n~exec\r\n\\`\\`\\`javascript\r\nconsole.log(\"hi!\");\r\n\\`\\`\\`"]
#[bucket = "exec_bucket"]
fn exec(ctx: &mut Context, msg: &Message) -> CommandResult {
    let arg = msg.content.clone();
    let split = arg.split("```");
    let data = ctx.data.read();
    let settings = data.get::<Settings>().unwrap().lock().unwrap().clone();

    let langs = data
        .get::<LangManager>()
        .unwrap()
        .lock()
        .unwrap()
        .get_languages_list();
    drop(data);

    if split.clone().nth(1).is_none() {
        let _ = msg.reply(&ctx, &format!("Please add a code section to your message\nExample:\n{}exec\n\\`\\`\\`language\n**code**\n\\`\\`\\`\nHere are the languages available: {}", settings.command_prefix, langs))?;
        return Ok(());
    }
    let code = split.take(2).collect::<Vec<_>>()[1];

    let mut split = code.split('\n');
    let (lang_code, code) = match split.next() {
        Some(line) => {
            let code = split.collect::<Vec<_>>().join("\n");
            let lang = line.trim().to_ascii_lowercase();
            (lang, code)
        }
        None => {
            let _ = msg.reply(
                &ctx,
                &format!(
                    ":x: Please specify a language\nHere are the languages available: {}",
                    langs
                ),
            )?;
            return Ok(());
        }
    };

    let mut reply_msg: Message;
    let (mut compilation, mut execution, lang) = {
        let lang = {
            // We make sure to lock the data in a separate code block,
            // Otherwise we would block the mutex through the entire compiling and/or executing phases
            let data = ctx.data.read();
            let mngr = data.get::<LangManager>().unwrap().lock().unwrap();
            get_lang(&mngr, lang_code.as_ref())
        };
        let lang = match lang {
            Ok(lang) => lang,
            Err(e) => {
                let _ = msg.reply(&ctx, &format!(":x: {}", e))?;
                return Ok(());
            }
        };

        {
            let data = ctx.data.read();
            let db = data.get::<DbPool>().unwrap();
            match models::Snippet::save(
                code.clone(),
                &lang.get_lang_name(),
                msg.author.id,
                msg.guild_id,
                db,
            ) {
                Ok(_) => {}
                Err(e) => warn!("Could not save snippet to db: {}", e),
            };
        }

        reply_msg = match msg.channel_id.say(&ctx, format!("<@{}>,", msg.author.id)) {
            Err(e) => {
                error!("An error occured while replying to an exec query: {}", e);
                return Ok(());
            }
            Ok(msg) => msg,
        };
        match run_code(
            &settings,
            code,
            lang.clone(),
            msg.author.id,
            Some(ctx),
            Some(&mut reply_msg),
        ) {
            Ok((compilation, execution, _processed_code, _lang_name)) => {
                (compilation, execution, lang)
            }
            Err(e) => {
                let _ = msg.reply(&ctx, &e.to_string())?;
                return Ok(());
            }
        }
    };

    compilation.stderr = pre_process_output(compilation.stderr);
    compilation.stdout = pre_process_output(compilation.stdout);
    execution.stderr = pre_process_output(execution.stderr);
    execution.stdout = pre_process_output(execution.stdout);

    {
        let data = ctx.data.read();
        let db = data.get::<DbPool>().unwrap();
        let mut stat = models::LangStat::get(&lang.get_lang_name(), db);
        stat.increment_snippets_count(db);
    }

    let header = format!("<@{}>,", msg.author.id);
    if let Err(why) = reply_msg.edit(ctx, |m| {
        m.content(header).embed(|mut e| {
            let mut fields_lang = vec![("Language", lang.get_lang_name(), true)];
            let mut fields_out = Vec::<(&str, String, bool)>::new();
            let mut color_red = false;

            let compil_t = (compilation.duration.as_millis() as f32) / 1000.0_f32;
            let exec_t = (execution.duration.as_millis() as f32) / 1000.0_f32;
            let mut fields_time = Vec::<(&str, String, bool)>::new();
            if compil_t > 0.0001 {
                fields_time.push(("Compilation time", format!("{:.1}s", compil_t), true));
            }
            if exec_t > 0.0001 {
                fields_time.push(("Execution time", format!("{:.1}s", exec_t), true));
            }

            if compilation.timed_out {
                // Compilation timed out
                e = e
                    .description(":x: Compilation timed out")
                    .colour(serenity::utils::Colour::RED);
                color_red = true;
            }
            if execution.timed_out {
                // Execution timed out
                e = e
                    .description(":x: Execution timed out")
                    .colour(serenity::utils::Colour::RED);
                color_red = true;
            }
            match compilation.exit_code {
                Some(code) if code != 0 => {
                    // Compilation failed
                    e = e
                        .description(":x: Compilation failed")
                        .colour(serenity::utils::Colour::RED);

                    let (truncated, out) = format_code_output(compilation.stderr, 1024);
                    let label = if truncated {
                        "Compilation error output (truncated)"
                    } else {
                        "Compilation error output"
                    };
                    fields_out.push((label, out, false));
                }
                _ => {
                    // Compilation succeeded
                    if !color_red {
                        e = e.colour(serenity::utils::Colour::DARK_GREEN);
                    }

                    if !compilation.stdout.is_empty() {
                        let (truncated, out) = format_code_output(compilation.stdout, 1024);
                        let label = if truncated {
                            "Compilation output (truncated)"
                        } else {
                            "Compilation output"
                        };
                        fields_out.push((label, out, false));
                    }
                    if !compilation.stderr.is_empty() {
                        if !color_red {
                            e = e.colour(serenity::utils::Colour::ORANGE);
                        }
                        let (truncated, out) = format_code_output(compilation.stderr, 1024);
                        let label = if truncated {
                            "Compilation error output (truncated)"
                        } else {
                            "Compilation error output"
                        };
                        fields_out.push((label, out, false));
                    }
                    if let Some(code) = execution.exit_code {
                        fields_lang.push(("Exit code", format!("`{}`", code), true));
                    }
                    if !execution.stdout.is_empty() {
                        let (truncated, out) = format_code_output(execution.stdout, 1024);
                        let label = if truncated {
                            "Standard output (truncated)"
                        } else {
                            "Standard output"
                        };
                        fields_out.push((label, out, false));
                    }
                    if !execution.stderr.is_empty() {
                        e = e.colour(serenity::utils::Colour::RED);
                        let (truncated, out) = format_code_output(execution.stderr, 1024);
                        let label = if truncated {
                            "Error output (truncated)"
                        } else {
                            "Error output"
                        };
                        fields_out.push((label, out, false));
                    }
                }
            };

            e = e.fields(fields_lang).fields(fields_time);
            if !fields_out.is_empty() {
                e = e.fields(fields_out)
            }

            e.author(|a| a.name(&msg.author.name).icon_url(&msg.author.face()))
                .timestamp(chrono::Utc::now().to_rfc3339())
                .thumbnail(lang.get_logo_url())
        })
    }) {
        error!(
            "An error occured while editing a reply to an exec query: {:?}",
            why
        );
        return Ok(());
    }

    info!("Done");
    Ok(())
}

fn format_code_output(mut text: String, max_length: usize) -> (bool, String) {
    let truncated = if text.len() > max_length - 7 {
        text.truncate(max_length - 7);
        true
    } else {
        false
    };

    text = format!("```\n{}```", text);
    (truncated, text)
}

fn get_random_filename(ext: &str) -> String {
    let mut rng = ::rand::thread_rng();
    let mut name: String;
    loop {
        name = iter::repeat(())
            .map(|_| rng.sample(Alphanumeric))
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

    Ok(path)
}

fn run_command(cmd: Expression, timeout_seconds: u64) -> Result<ExecResult, Error> {
    let child = cmd
        .unchecked() // important! allows us to get stderr instead of an `Error` if the process exits with a non-zero exit code
        .stdout_capture()
        .stderr_capture()
        .start()?;

    let timeout = Duration::from_secs(timeout_seconds);
    let start = Instant::now();

    loop {
        if child.try_wait()?.is_some() {
            break;
        }

        if timeout_seconds != 0 && start.elapsed() >= timeout {
            child.kill()?;

            return Ok(ExecResult {
                exit_code: None,
                stdout: "".into(),
                stderr: "".into(),
                timed_out: true,
                duration: start.elapsed(),
            });
        }

        ::std::thread::sleep(Duration::from_millis(250));
    }

    let output = child.wait()?;

    let stdout = ::std::str::from_utf8(&output.stdout)
        .map_err(|e| Error::new(ErrorKind::InvalidData, e))?
        .to_owned();
    let stderr = ::std::str::from_utf8(&output.stderr)
        .map_err(|e| Error::new(ErrorKind::InvalidData, e))?
        .to_owned();

    Ok(ExecResult {
        exit_code: output.status.code(),
        stdout,
        stderr,
        timed_out: false,
        duration: start.elapsed(),
    })
}
