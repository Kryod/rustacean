use crate::LangManager;

use serenity::{
    prelude::Context,
    model::channel::Message,
    framework::standard::{ CommandResult, macros::command },
};


#[command]
#[aliases("langs", "language", "lang")]
#[description = "Get a list of available programming languages for the `exec` command."]
fn languages(ctx: &mut Context, msg: &Message) -> CommandResult {
    let data = ctx.data.read();
    let lang_manager = data.get::<LangManager>().unwrap().lock().unwrap();
    let mut fields: Vec<(String, String, bool)> = Vec::new();
    for (lang_codes, boxed_lang) in lang_manager.get_languages() {
        if lang_manager.is_language_available(&(*boxed_lang)) {
            fields.push((
                boxed_lang.get_lang_name(),
                format!("({})", lang_codes.iter().map(|code| format!("`{}`", code)).collect::<Vec<String>>().join(", ")),
                true
            ));
        }
    }
    fields.sort();

    let _ = msg.channel_id.send_message(&ctx, |m| m
        .embed(|e| e
            .title("Languages")
            .description("A list of available languages for the `exec` command.")
            .fields(fields)
        )
    )?;
    Ok(())
}
