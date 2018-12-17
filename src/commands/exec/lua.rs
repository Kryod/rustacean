use std::path::PathBuf;

use commands::exec::language::Language;
use duct::{ cmd, Expression };

#[derive(Debug)]
pub struct Lua;

impl Language for Lua {
    fn get_lang_name(&self) -> String {
        "Lua".into()
    }

    fn get_source_file_ext(&self) -> String {
        ".lua".into()
    }

    fn get_execution_command(&self, path: &PathBuf) -> Expression {
        cmd!("lua", path)    
    }
}
