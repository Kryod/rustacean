use serenity::{
    prelude::Context,
    model::channel::Message,
    framework::standard::{ CommandResult, macros::command },
};

#[command]
#[description = "Get an invite link to add Rustacean to other servers."]
fn invite(ctx: &mut Context, msg: &Message) -> CommandResult {
    let user = match ctx.http.get_current_user() {
        Ok(user) => user,
        Err(_) => {
            let _ = msg.reply(ctx, "An error occurred.");
            return Ok(());
        }
    };
    let link = format!("https://discordapp.com/oauth2/authorize?client_id={}&scope=bot&permissions=378944", user.id);
    let _ = msg.reply(ctx, &format!("Use this link to invite me to another server: {}", link))?;
    Ok(())
}
