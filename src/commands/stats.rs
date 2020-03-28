use crate::{ models, LangManager, DbPool };

use serenity::{
    prelude::Context,
    model::channel::Message,
    framework::standard::{ CommandResult, macros::command },
};

#[command]
#[aliases("stat")]
#[description = "Gets statistics about usage of languages for the `exec` command."]
fn stats(ctx: &mut Context, msg: &Message) -> CommandResult {
    let data = ctx.data.read();
    let lang_manager = data.get::<LangManager>().unwrap().lock().unwrap();
    let db = data.get::<DbPool>().unwrap();
    let mut fields: Vec<(String, String, bool)> = Vec::new();
    for boxed_lang in lang_manager.get_languages().values() {
        if lang_manager.is_language_available(&(*boxed_lang)) {
            fields.push((
                boxed_lang.get_lang_name(),
                models::LangStat::get(&boxed_lang.get_lang_name(), db).get_snippets_executed().to_string(),
                true
            ));
        }
    }
    fields.sort();

    let _ = msg.channel_id.send_message(&ctx, |m| m
        .embed(|e| e
            .title("Stats")
            .description("Stats about usage of each language.")
            .fields(fields)
        )
    )?;

    Ok(())
}