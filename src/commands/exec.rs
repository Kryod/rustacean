command!(exec(_ctx, msg, _args) {
    let arg = msg.content.clone();
    let mut code = arg.split("```")
            .take(2)
            .collect::<Vec<_>>()[1];

    if code.get(0..2) != Some("rs") {
        let _ = msg.channel_id.say("PLIZ CODE IN RUST!");
    } else {
        code = code.get(2 .. code.len()).unwrap();
        let mut one = String::from("https://play.integer32.com/?code=");
        one = format!("{}{}", one, code).replace("{", "%7B")
            .replace("}", "%7D")
            .replace("\n", "%0A")
            .replace(" ", "%20");
        let _ = msg.channel_id.say(one);
    }
});