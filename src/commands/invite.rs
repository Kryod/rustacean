command!(invite(_ctx, msg, _args) {
    let user = match serenity::http::raw::get_current_user() {
        Ok(user) => user,
        Err(_) => {
            let _ = msg.reply("An error occurred.");
            return Ok(());
        }
    };
    let link = format!("https://discordapp.com/oauth2/authorize?client_id={}&scope=bot&permissions=378944", user.id);
    let _ = msg.reply(&format!("Use this link to invite me to another server: {}", link));
});
