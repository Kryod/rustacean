use std::path::PathBuf;

use commands::exec::language::Language;
use duct::{ cmd, Expression };

#[derive(Debug)]
pub struct Lua;

impl Lua {
    fn get_interpreter(&self) -> String {
        if cfg!(windows) {
            "lua53".into()
        } else {
            "lua5.3".into()
        }
    }
}

impl Language for Lua {
    fn get_lang_name(&self) -> String {
        "Lua".into()
    }

    fn get_source_file_ext(&self) -> String {
        ".lua".into()
    }

    fn get_execution_command(&self, path: &PathBuf) -> Expression {
        cmd!(self.get_interpreter(), path)
    }

    fn check_compiler_or_interpreter(&self) -> Expression {
        cmd!(self.get_interpreter(), "-v")
    }
}
