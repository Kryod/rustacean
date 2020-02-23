use crate::LangManager;

command!(languages(ctx, msg, _args) {
    let mut data = ctx.data.lock();
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

    let _ = msg.channel_id.send_message(|m| m
        .embed(|e| e
            .title("Languages")
            .description("A list of available languages for the `exec` command.")
            .fields(fields)
        )
    );
});
