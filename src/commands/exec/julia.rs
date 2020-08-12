use std::path::PathBuf;

use crate::commands::exec::language::Language;

#[derive(Debug)]
pub struct Julia;

impl Language for Julia {
    fn get_image_name(&self) -> String {
        "rustacean-julia".into()
    }

    fn get_lang_name(&self) -> String {
        "Julia".into()
    }

    fn get_source_file_ext(&self) -> String {
        ".jl".into()
    }

    fn get_execution_command(&self, path: &PathBuf) -> String {
        format!("julia {}", path.to_str().unwrap())
    }

    fn check_compiler_or_interpreter(&self) -> String {
        String::from("julia --version")
    }
}
