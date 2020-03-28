use duct::cmd;
use serenity::{
    prelude::Context,
    model::channel::Message,
    framework::standard::{ Args, CommandResult, macros::command },
};

#[derive(Debug)]
enum Type {
    INFO,
    DEBUG,
    ERROR,
}

impl Type {
    pub fn from_str(s: &str) -> Option<Type> {
        match s {
            "INFO" => Some(Type::INFO),
            "DEBUG" => Some(Type::DEBUG),
            "ERROR" => Some(Type::ERROR),
            _ => None,
        }
    }
}

fn parsing(args: Vec<String>) -> (Option<i64>, Option<String>) {
    let mut types = None;
    let mut lines = None;

    for arg in args
    {

        match Type::from_str(arg.trim()) {
            Some(Type::INFO) | Some(Type::DEBUG) | Some(Type::ERROR) => types = Some(arg.clone()),
            None => {}
        }

        let int_parse = arg.trim().parse::<i64>();
        if let Ok(i) = int_parse {
            lines = Some(i);
        }
    }

    (lines, types) 
}

#[command]
#[description = "Returns lines from the current log file. You can specify a string to search (INFO, DEBUG, ...). By default it gives the last 11 lines."]
#[example = "20 INFO"]
#[owners_only]
fn logs(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
    let mut args_vec: Vec<String> = Vec::new();
    for arg in args.iter::<String>() {
        args_vec.push(arg.unwrap_or_else(|_| "".to_string()));
    }

    let (lines, types) = parsing(args_vec);

    let grep = types.is_some();
    let nb = lines.is_some();

    let log = match (grep, nb) {
        (false, false) => {
                cmd!("tail", "/home/rustacean.log")
                .stdout_capture()
                .read()
                .unwrap()
            },
        (true, false) => {
                cmd!("cat", "/home/rustacean.log")
                .pipe(cmd!("grep", types.unwrap()))
                .pipe(cmd!("tail"))
                .stdout_capture()
                .read()
                .unwrap()
            },
        (true, true) => {
                cmd!("cat", "/home/rustacean.log")
                .pipe(cmd!("grep", types.unwrap()))
                .pipe(cmd!("tail", "-n", lines.unwrap().to_string(),))
                .stdout_capture()
                .read()
                .unwrap()
            },
        (false, true) => {
                cmd!("tail", "-n", lines.unwrap().to_string(), "/home/rustacean.log")
                .stdout_capture()
                .read()
                .unwrap()
            },
    };

    if log != "" {
        let _ = msg.reply(ctx, &format!("```{}```", log))?;
    } else {
        let _ = msg.reply(ctx, "Could not find anything")?;
    }

    Ok(())
}
