use std::path::PathBuf;

use commands::exec::language::Language;
use duct::{ cmd, Expression };

#[derive(Debug)]
pub struct JavaScript;

impl JavaScript {
    fn get_interpreter(&self) -> String {
        if cfg!(windows) {
            "node".into()
        } else if cfg!(target_os="macos") {
            "node".into()
        } else {
            "nodejs".into()
        }
    }
}

impl Language for JavaScript {
    fn get_lang_name(&self) -> String {
        "JavaScript".into()
    }

    fn get_source_file_ext(&self) -> String {
        ".js".into()
    }

    fn get_execution_command(&self, path: &PathBuf) -> Expression {
        cmd!(self.get_interpreter(), path)
    }

    fn check_compiler_or_interpreter(&self) -> Expression {
        cmd!(self.get_interpreter(), "--version")
    }
}
