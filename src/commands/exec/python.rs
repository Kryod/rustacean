use ::Language;
use std::path::PathBuf;
use duct::{ cmd, Expression };

#[derive(Debug)]
pub struct Python;

impl Language for Python {
    fn get_lang_name(&self) -> String {
        "Python".into()
    }

    fn get_source_file_ext(&self) -> String {
        ".py".into()
    }

    fn get_execution_command(&self, path: PathBuf) -> Expression {
        cmd!("python3", path)
    }
}
