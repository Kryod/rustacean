use std::path::PathBuf;

use commands::exec::language::Language;
use duct::{ cmd, Expression };

#[derive(Debug)]
pub struct Shell;

impl Language for Shell {
    fn get_image_name(&self) -> String {
        "rustacean-shell".into()
    }
    
    fn get_lang_name(&self) -> String {
        "Shell".into()
    }

    fn get_source_file_ext(&self) -> String {
        ".sh".into()
    }

    fn get_execution_command(&self, path: &PathBuf) -> String {
        format!("sh {}", path.to_str().unwrap())    
    }

    fn check_compiler_or_interpreter(&self) -> Expression {
        cmd!("sh", "-c", "echo \"ok\"")
    }
}
