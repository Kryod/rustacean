use std::path::PathBuf;

use crate::commands::exec::language::Language;

#[derive(Debug)]
pub struct Typescript;

impl Language for Typescript {
    fn get_image_name(&self) -> String {
        "rustacean-typescript".into()
    }

    fn get_lang_name(&self) -> String {
        "TypeScript".into()
    }

    fn get_source_file_ext(&self) -> String {
        ".ts".into()
    }

    fn get_compiler_command(&self, src_path: &PathBuf, exe_path: &PathBuf) -> Option<String> {
        Some(format!(
            "tsc {} --outFile {}.js",
            src_path.to_str().unwrap(),
            exe_path.to_str().unwrap()
        ))
    }

    fn get_execution_command(&self, path: &PathBuf) -> String {
        format!("node {}.js", path.to_str().unwrap())
    }

    fn check_compiler_or_interpreter(&self) -> String {
        String::from("tsc -v")
    }
}
