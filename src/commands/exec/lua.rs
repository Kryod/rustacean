use std::path::PathBuf;

use crate::commands::exec::language::Language;

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
    fn get_image_name(&self) -> String {
        "rustacean-lua".into()
    }

    fn get_lang_name(&self) -> String {
        "Lua".into()
    }

    fn get_source_file_ext(&self) -> String {
        ".lua".into()
    }

    fn get_execution_command(&self, path: &PathBuf) -> String {
        format!("{} {}", self.get_interpreter(), path.to_str().unwrap())
    }

    fn check_compiler_or_interpreter(&self) -> String {
        format!("{} -v",self.get_interpreter())
    }
}
