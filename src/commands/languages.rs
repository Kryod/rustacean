command!(languages(ctx, msg, _args) {
    let mut data = ctx.data.lock();
    let lang_manager = data.get::<::LangManager>().unwrap();
    let mut fields: Vec<(String, String, bool)> = Vec::new();
    for (lang_codes, boxed_lang) in lang_manager {
        fields.push((
            boxed_lang.get_lang_name(),
            format!("({})", lang_codes.iter().map(|code| format!("`{}`", code)).collect::<Vec<String>>().join(", ")),
            true
        ));
    }

    let _ = msg.channel_id.send_message(|m| m
        .embed(|e| e
            .title("Languages")
            .description("A list of available languages for the `exec` command.")
            .fields(fields)
        )
    );
});
