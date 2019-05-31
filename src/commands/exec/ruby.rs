use std::path::PathBuf;

use commands::exec::language::Language;
use duct::{ cmd, Expression };

#[derive(Debug)]
pub struct Ruby;

impl Ruby {
    fn get_interpreter(&self) -> String {
        "ruby".into()
    }
}

impl Language for Ruby {
    fn get_image_name(&self) -> String {
        "rustacean-ruby".into()
    }

    fn get_lang_name(&self) -> String {
        "Ruby".into()
    }

    fn get_source_file_ext(&self) -> String {
        ".rb".into()
    }

    fn get_execution_command(&self, path: &PathBuf) -> String {
        format!("{} {}", self.get_interpreter(), path.to_str().unwrap())
    }

    fn check_compiler_or_interpreter(&self) -> Expression {
        cmd!(self.get_interpreter(), "--version")
    }
}
