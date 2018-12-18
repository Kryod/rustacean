use typemap::Key;
use std::sync::{ Arc, Mutex };
use std::collections::HashMap;

use commands::exec::language::Language;
use commands::exec::*;

pub struct LangManager {
    languages: HashMap<Vec<String>, Arc<Box<Language + Sync + Send>>>,
    availability: HashMap<String, bool>,
}

impl Key for LangManager {
    type Value = Arc<Mutex<LangManager>>;
}

impl LangManager {
    pub fn new() -> LangManager {
        let mut mngr = LangManager {
            languages: HashMap::new(),
            availability: HashMap::new(),
        };

        mngr.languages.insert(vec![
            "rust".into(),
            "rs".into()
        ], Arc::new(Box::new(Rust)));

        mngr.languages.insert(vec!["c".into()], Arc::new(Box::new(C)));
        mngr.languages.insert(vec!["cpp".into()], Arc::new(Box::new(Cpp)));
        mngr.languages.insert(vec!["php".into()], Arc::new(Box::new(Php)));
        mngr.languages.insert(vec!["lua".into()], Arc::new(Box::new(Lua)));
        mngr.languages.insert(vec!["asm".into()], Arc::new(Box::new(Asm)));
        mngr.languages.insert(vec![
            "sh".into(),
            "shell".into()
            ], Arc::new(Box::new(Shell)));
        mngr.languages.insert(vec![
            "py".into(),
            "python".into(),
        ], Arc::new(Box::new(Python)));
        mngr.languages.insert(vec![
            "js".into(),
            "javascript".into(),
        ], Arc::new(Box::new(JavaScript)));
        mngr.languages.insert(vec![
            "cs".into(),
            "csharp".into(),
        ], Arc::new(Box::new(Csharp)));
        mngr.languages.insert(vec![
            "vb".into(),
            "vbnet".into(),
        ], Arc::new(Box::new(Vb)));
        mngr.languages.insert(vec!["java".into()], Arc::new(Box::new(Java)));

        mngr.check_compilers_interpreters();

        mngr
    }

    pub fn get(&self, lang: &String) -> Option<Arc<Box<Language + Sync + Send>>> {
        for (lang_codes, boxed_lang) in self.languages.iter() {
            for l in lang_codes {
                if l == lang {
                    return Some(boxed_lang.clone())
                }
            }
        }
        None
    }

    pub fn is_language_available(&self, lang: &Box<Language + std::marker::Sync + std::marker::Send>) -> bool {
        match self.availability.get(&lang.get_lang_name()) {
            Some(availability) => *availability,
            None => false,
        }
    }

    pub fn get_languages_list(&self) -> String {
        let mut langs: Vec<String> = Vec::new();
        for (lang_codes, boxed_lang) in self.languages.iter() {
            for lang in lang_codes {
                if self.is_language_available(&(*boxed_lang)) {
                    langs.push(lang.clone());
                }
            }
        }
        langs.sort_by(|a, b| a.cmp(b));
        langs.join(", ")
    }

    pub fn get_languages(&self) -> &HashMap<Vec<String>, Arc<Box<Language + Sync + Send>>> {
        &self.languages
    }

    fn check_compilers_interpreters(&mut self) {
        info!("Checking available languages...");
        for (_lang_codes, boxed_lang) in self.languages.iter() {
            let command = boxed_lang.check_compiler_or_interpreter().stdout_null().stderr_null();
            let lang_name = boxed_lang.get_lang_name();
            self.availability.insert(lang_name.clone(), false);
            match command.run() {
                Ok(res) => {
                    if res.status.success() {
                        info!("    - {}: Available", &lang_name);
                        self.availability.insert(lang_name, true);
                    } else {
                        warn!("    - {}: Unavailable", &lang_name);
                    }
                },
                Err(e) => {
                    warn!("    - {}: Unavailable ({})", &lang_name, e);
                }
            };
        }
    }
}
