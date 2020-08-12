use std::path::PathBuf;

use crate::commands::exec::language::Language;

#[derive(Debug)]
pub struct Haskell;

impl Language for Haskell {
    fn get_image_name(&self) -> String {
        "rustacean-haskell".into()
    }

    fn get_lang_name(&self) -> String {
        "Haskell".into()
    }

    fn get_source_file_ext(&self) -> String {
        ".hs".into()
    }

    fn get_compiler_command(&self, src_path: &PathBuf, exe_path: &PathBuf) -> Option<String> {
        Some(format!(
            "ghc -o {} {}",
            exe_path.to_str().unwrap(),
            src_path.to_str().unwrap()
        ))
    }

    fn check_compiler_or_interpreter(&self) -> String {
        String::from("ghc --version")
    }
}
