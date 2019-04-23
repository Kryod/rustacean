use std::path::PathBuf;

use commands::exec::language::Language;
use duct::{ cmd, Expression };

#[derive(Debug)]
pub struct Cpp;

impl Language for Cpp {

    fn get_image_name(&self) -> String {
        "rustacean-c++".into()
    }

    fn get_lang_name(&self) -> String {
        "C++".into()
    }

    fn get_source_file_ext(&self) -> String {
        ".cpp".into()
    }

    fn pre_process_code(&self, code: &str, _src_path: &PathBuf) -> Option<String> {
        use regex::Regex;

        let re = Regex::new(r"int\s*main\s*\(.*\)").unwrap();
        if !re.is_match(&code) {
            let result = format!("#include <iostream>\r\nint main(int argc, char* argv[]) {{\r\n{}\r\n}}", code);
            return Some(result);
        }

        None
    }

    fn get_compiler_command(&self, src_path: &PathBuf, exe_path: &PathBuf) -> Option<String> {
        Some(format!("g++ {} -o {}", src_path.to_str().unwrap(), exe_path.to_str().unwrap()))
    }

    fn check_compiler_or_interpreter(&self) -> Expression {
        cmd!("g++", "--version")
    }
}
