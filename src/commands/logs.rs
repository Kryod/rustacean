use duct::cmd;

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
        match int_parse {
            Ok(i) => lines = Some(i),
            Err(_) => {}
        }
    }

    (lines, types) 
}

command!(logs(_ctx, msg, args) {

    let mut args_vec: Vec<String> = Vec::new();
    for arg in args.iter::<String>() {
        args_vec.push(arg.unwrap_or("".to_string()));
    }

    let (lines, types) = parsing(args_vec);

    let grep = match types {
        Some(_) => true,
        None => false,
    };

    let nb = match lines {
        Some(_) => true,
        None => false,
    };

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
        let _ = msg.reply(&format!("```{}```", log));
    } else {
        let _ = msg.reply(&format!("Could not find anything"));
    }
});
