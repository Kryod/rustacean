use duct::{ cmd, Expression };

command!(logs(_ctx, msg, _args) {
    //if args == ""
    let log = cmd!("tail", "/home/rustacean.log").stdout_capture().read().unwrap();
    let _ = msg.reply(&format!("```{}```", log));
});
