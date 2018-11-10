command!(languages(ctx, msg, _args) {
    let mut data = ctx.data.lock();
    let lang_manager = data.get::<::LangManager>().unwrap();
    let mut langs: Vec<String> = Vec::new();
    for lang in lang_manager.keys() {
        langs.push(lang.clone());
    }
    langs.sort_by(|a, b| a.cmp(b));
    let mut langs_str = String::from("");
    for lang in langs {
        langs_str += &format!("\t- {}\n", lang);
    }

    let _ = msg.channel_id.say(format!("Here are the languages available:\n{}", langs_str));
});
