use std::path::PathBuf;

use commands::exec::language::Language;
use duct::{ cmd, Expression };

#[derive(Debug)]
pub struct Shell;

impl Language for Shell {
    fn get_lang_name(&self) -> String {
        "Shell".into()
    }

    fn get_source_file_ext(&self) -> String {
        ".sh".into()
    }

    fn get_execution_command(&self, path: &PathBuf) -> Expression {
        cmd!("sh", path)    
    }
}
