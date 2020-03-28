use serenity::{
    prelude::Context,
    model::channel::Message,
    framework::standard::{ CommandResult, macros::command },
};

#[command]
#[description = "Get a link to Rustacean's support Discord server."]
fn support(ctx: &mut Context, msg: &Message) -> CommandResult {
    let _ = msg.reply(ctx, "Join Rustacean's support server here: https://discordapp.com/invite/2qjtv2H")?;
    Ok(())
}
