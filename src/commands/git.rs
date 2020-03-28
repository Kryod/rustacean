use serenity::{
    prelude::Context,
    model::channel::Message,
    framework::standard::{ CommandResult, macros::command },
};

#[command]
#[aliases("github", "repository", "repo")]
#[description = "Get a link to Rustacean's GitHub repository."]
fn git(ctx: &mut Context, msg: &Message) -> CommandResult {
    let _ = msg.reply(ctx, "Here is the GitHub repository: https://github.com/Kryod/rustacean, you can contribute via pull requests and submit issues.");
    Ok(())
}
