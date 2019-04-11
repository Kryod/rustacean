command!(versions(ctx, msg, _args) {
    let mut data = ctx.data.lock();
    let lang_manager = data.get::<::LangManager>().unwrap().lock().unwrap();
    let mut fields: Vec<(String, String, bool)> = Vec::new();
    for (_lang_codes, boxed_lang) in lang_manager.get_languages() {
        if lang_manager.is_language_available(&(*boxed_lang)) {
            fields.push((
                boxed_lang.get_lang_name(),
                boxed_lang.check_compiler_or_interpreter().stdout_capture().read().unwrap(),
                true
            ));
        }
    }
    fields.sort();

    let _ = msg.channel_id.send_message(|m| m
        .embed(|e| e
            .title("Versions")
            .description("A list of versions of languages available.")
            .fields(fields)
        )
    );
});
