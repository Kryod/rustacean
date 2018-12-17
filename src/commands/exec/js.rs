use ::Language;
use std::path::PathBuf;
use duct::{ cmd, Expression };

#[derive(Debug)]
pub struct JavaScript;

impl Language for JavaScript {
    fn get_lang_name(&self) -> String {
        "JavaScript".into()
    }

    fn get_source_file_ext(&self) -> String {
        ".js".into()
    }

    fn get_execution_command(&self, path: &PathBuf) -> Expression {
        if cfg!(windows) {
            cmd!("node", path)
        } else if cfg!(target_os="macos") {
            cmd!("node", path)
        } else {
            cmd!("nodejs", path)
        }
    }
}
