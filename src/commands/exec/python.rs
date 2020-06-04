use std::path::PathBuf;

use crate::commands::exec::language::Language;

#[derive(Debug)]
pub struct Python;

impl Python {
    fn get_interpreter(&self) -> String {
        if cfg!(windows) {
            "python".into()
        } else {
            "python3".into()
        }
    }
}

impl Language for Python {
    fn get_image_name(&self) -> String {
        "rustacean-python".into()
    }

    fn get_lang_name(&self) -> String {
        "Python".into()
    }

    fn get_source_file_ext(&self) -> String {
        ".py".into()
    }

    fn get_execution_command(&self, path: &PathBuf) -> String {
        format!("{} {}",self.get_interpreter(), path.to_str().unwrap())
    }

    fn check_compiler_or_interpreter(&self) -> String {
        format!("{} --version",self.get_interpreter())
    }
}
