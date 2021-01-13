use std::path::PathBuf;

use crate::commands::exec::language::Language;

#[derive(Debug)]
pub struct Dyalog;

impl Language for Dyalog {
    fn get_image_name(&self) -> String {
        "rustacean-apl".into()
    }

    fn get_lang_name(&self) -> String {
        "APL".into()
    }

    fn get_source_file_ext(&self) -> String {
        ".apl".into()
    }

    fn get_compiler_command(&self, src_path: &PathBuf, exe_path: &PathBuf) -> Option<String> {
        Some(format!(
            "aplc -o {} {}",
            exe_path.to_str().unwrap(),
            src_path.to_str().unwrap()
        ))
    }

    fn check_compiler_or_interpreter(&self) -> String {
        String::from("aplc --version")
    }
}
