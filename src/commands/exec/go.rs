use std::path::PathBuf;

use crate::commands::exec::language::Language;

#[derive(Debug)]
pub struct Go;

impl Language for Go {
    fn get_image_name(&self) -> String {
        "rustacean-go".into()
    }

    fn get_lang_name(&self) -> String {
        "Go".into()
    }

    fn get_source_file_ext(&self) -> String {
        ".go".into()
    }

    fn get_compiler_command(&self, _src_path: &PathBuf, exe_path: &PathBuf) -> Option<String> {
        Some(format!("go build -o {}", exe_path.to_str().unwrap()))
    }

    fn check_compiler_or_interpreter(&self) -> String {
        String::from("go version")
    }
}
