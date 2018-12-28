use duct::cmd;

command!(logs(_ctx, msg, args) {

    
    let mut lines = args.single::<i64>();
    let mut types = args.single::<String>();
    
    let mut grep: bool = false;
    let mut nb: bool = false;

    let types = match types {
        Ok(ref types) if types.is_empty() => { 
            grep = false;
            "".to_string()
        },
        Ok(types) => { 
            grep = true;
            types
        },
        Err(_) => { 
            grep = false;
            "".to_string()
        },
    };

    let lines = match lines {
        Ok(lines) if lines == 0 => { 
            nb = false;
            0
        },
        Ok(lines) => { 
            nb = true;
            lines
        },
        Err(_) => { 
            nb = false;
            0
        },
    };

    let log = match (grep, nb) {
        (false, false) => cmd!("tail", "/home/rustacean.log").stdout_capture().read().unwrap(),
        (true, false) => cmd!("tail", "/home/rustacean.log").pipe(cmd!("grep", types).unchecked()).stdout_capture().read().unwrap(),
        (true, true) => cmd!("tail", "-n", lines.to_string(), "/home/rustacean.log").pipe(cmd!("grep", types).unchecked()).stdout_capture().read().unwrap(),
        (false, true) => cmd!("tail", "-n", lines.to_string(), "/home/rustacean.log").stdout_capture().read().unwrap(),
    };

    /*(Ok(""), Ok(0))  => log = cmd!("tail", "/home/rustacean.log").stdout_capture().read().unwrap(),
        (Ok(types), Ok(0)) => log = cmd!("tail", "/home/rustacean.log").pipe(cmd!("grep", types).unchecked()).stdout_capture().read().unwrap(),
        (Ok(""), Ok(lines)) => log = cmd!("tail", "-n", lines.to_string(), "/home/rustacean.log").stdout_capture().read().unwrap(),
        (Ok(types), Ok(lines)) => log = cmd!("tail", "-n", lines.to_string(), "/home/rustacean.log").pipe(cmd!("grep", types).unchecked()).stdout_capture().read().unwrap(),*/
    //let log = cmd!("tail", "/home/rustacean.log").stdout_capture().read().unwrap();
    if (log != "") {
        let _ = msg.reply(&format!("```{}```", log));
    } else {
        let _ = msg.reply(&format!("Grep didn't find anything"));
    }
});
