use std::path::PathBuf;

use duct::{ cmd, Expression };

use crate::commands::exec::language::Language;

#[derive(Debug)]
pub struct JavaScript;

impl JavaScript {
    fn get_interpreter(&self) -> String {
        if cfg!(windows) || cfg!(target_os="macos") {
            "node".into()
        } else {
            "nodejs".into()
        }
    }
}

impl Language for JavaScript {
    fn get_image_name(&self) -> String {
        "rustacean-javascript".into()
    }

    fn get_lang_name(&self) -> String {
        "JavaScript".into()
    }

    fn get_source_file_ext(&self) -> String {
        ".js".into()
    }

    fn get_execution_command(&self, path: &PathBuf) -> String {
        format!("{} {}", self.get_interpreter(), path.to_str().unwrap())
    }

    fn check_compiler_or_interpreter(&self) -> Expression {
        cmd!(self.get_interpreter(), "--version")
    }
}
