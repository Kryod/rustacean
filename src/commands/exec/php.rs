use std::path::PathBuf;

use commands::exec::language::Language;
use duct::{ cmd, Expression };

#[derive(Debug)]
pub struct Php;

impl Language for Php {
    fn get_image_name(&self) -> String {
        "rustacean-php".into()
    }
    
    fn get_lang_name(&self) -> String {
        "PHP".into()
    }

    fn get_source_file_ext(&self) -> String {
        ".php".into()
    }

    fn get_execution_command(&self, path: &PathBuf) -> String {
        format!("php {}", path.to_str().unwrap())
    }

    fn check_compiler_or_interpreter(&self) -> Expression {
        cmd!("php", "--version")
    }
}
