command!(stats(ctx, msg, _args) {
    let mut data = ctx.data.lock();
    let lang_manager = data.get::<::LangManager>().unwrap().lock().unwrap();
    let db = data.get::<::DbPool>().unwrap();
    let mut fields: Vec<(String, String, bool)> = Vec::new();
    for (_lang_codes, boxed_lang) in lang_manager.get_languages() {
        if lang_manager.is_language_available(&(*boxed_lang)) {
            fields.push((
                boxed_lang.get_lang_name(),
                ::models::LangStat::get(&boxed_lang.get_lang_name(), db).get_snippets_executed().to_string(),
                true
            ));
        }
    }
    fields.sort();

    let _ = msg.channel_id.send_message(|m| m
        .embed(|e| e
            .title("Stats")
            .description("Stats about usage of each language.")
            .fields(fields)
        )
    );
});