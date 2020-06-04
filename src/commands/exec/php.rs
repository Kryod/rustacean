use std::path::PathBuf;

use crate::commands::exec::language::Language;

#[derive(Debug)]
pub struct Php;

impl Language for Php {
    fn get_image_name(&self) -> String {
        "rustacean-php".into()
    }

    fn get_lang_name(&self) -> String {
        "PHP".into()
    }

    fn get_source_file_ext(&self) -> String {
        ".php".into()
    }

    fn pre_process_code(&self, code: &str, _src_path: &PathBuf) -> Option<String> {
        let re = regex::Regex::new(r"(<\?php|<\?=)").unwrap();
        if !re.is_match(&code) {
            let result = format!("<?php\r\n{}", code);
            return Some(result);
        }

        None
    }

    fn get_execution_command(&self, path: &PathBuf) -> String {
        format!("php {}", path.to_str().unwrap())
    }

    fn check_compiler_or_interpreter(&self) -> String {
        String::from("php --version")
    }
}
