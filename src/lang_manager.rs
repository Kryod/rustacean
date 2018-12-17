use typemap::Key;
use std::sync::Arc;
use std::collections::HashMap;

use commands::exec::language::Language;
use commands::exec::*;

pub struct LangManager;
pub type LangManagerType = HashMap<Vec<String>, Arc<Box<Language + Sync + Send>>>;

impl Key for LangManager {
    type Value = LangManagerType;
}

impl LangManager {
    pub fn default() -> LangManagerType {
        let mut langs: LangManagerType = HashMap::new();

        langs.insert(vec![
            "rust".into(),
            "rs".into()
        ], Arc::new(Box::new(Rust)));
        langs.insert(vec!["c".into()], Arc::new(Box::new(C)));
        langs.insert(vec!["cpp".into()], Arc::new(Box::new(Cpp)));
        langs.insert(vec!["php".into()], Arc::new(Box::new(Php)));
        langs.insert(vec!["lua".into()], Arc::new(Box::new(Lua)));
        langs.insert(vec!["asm".into()], Arc::new(Box::new(Asm)));
        langs.insert(vec![
            "sh".into(),
            "shell".into()
            ], Arc::new(Box::new(Shell)));
        langs.insert(vec![
            "py".into(),
            "python".into(),
        ], Arc::new(Box::new(Python)));
        langs.insert(vec![
            "js".into(),
            "javascript".into(),
        ], Arc::new(Box::new(JavaScript)));
        langs.insert(vec![
            "cs".into(),
            "csharp".into(),
        ], Arc::new(Box::new(Csharp)));
        langs.insert(vec!["java".into()], Arc::new(Box::new(Java)));

        langs
    }

    pub fn get(mngr: &LangManagerType, lang: &String) -> Option<Arc<Box<Language + Sync + Send>>> {
        for (lang_codes, boxed_lang) in mngr.iter() {
            for l in lang_codes {
                if l == lang {
                    return Some(boxed_lang.clone())
                }
            }
        }
        None
    }

    pub fn get_langs(mngr: &LangManagerType) -> String {
        let mut langs: Vec<String> = Vec::new();
        for lang_codes in mngr.keys() {
            for lang in lang_codes {
                langs.push(lang.clone());
            }
        }
        langs.sort_by(|a, b| a.cmp(b));
        langs.join(", ")
    }
}
