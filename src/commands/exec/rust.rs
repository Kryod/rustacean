use ::Language;
use std::path::PathBuf;
use duct::{ cmd, Expression };

#[derive(Debug)]
pub struct Rust;

impl Language for Rust {
    fn get_lang_name(&self) -> String {
        "Rust".into()
    }

    fn get_source_file_ext(&self) -> String {
        ".rs".into()
    }

    fn pre_process_code(&self, code: &str, _src_path: &PathBuf) -> Option<String> {
        use regex::Regex;

        let re = Regex::new(r"fn\s*main\s*\(\s*\)").unwrap();
        if !re.is_match(&code) {
            let result = format!("fn main() {{\r\n{}\r\n}}", code);
            return Some(result);
        }

        None
    }

    fn get_compiler_command(&self, src_path: &PathBuf, exe_path: &PathBuf) -> Option<Expression> {
        Some(cmd!("rustc", src_path, "-o", exe_path))
    }
}
