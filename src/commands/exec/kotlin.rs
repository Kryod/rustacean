use std::path::PathBuf;

use commands::exec::language::Language;
use duct::{ cmd, Expression };

#[derive(Debug)]
pub struct Kotlin;

impl Kotlin {
    fn get_class_name(&self, src_path: &PathBuf) -> String {
        src_path.with_extension("").file_name().unwrap().to_str().unwrap().into()
    }

    fn get_compiler(&self) -> String {
        if cfg!(windows) {
            "kotlinc.bat".into()
        } else {
            "kotlinc".into()
        }
    }
}

impl Language for Kotlin {
    fn get_image_name(&self) -> String {
        "rustacean-kotlin".into()
    }
    
    fn get_lang_name(&self) -> String {
        "Kotlin".into()
    }

    fn get_source_file_ext(&self) -> String {
        ".kt".into()
    }

    fn pre_process_code(&self, code: &str, _src_path: &PathBuf) -> Option<String> {
        use regex::Regex;

        let re = Regex::new(r"(?s)(fun\s+main\s*\(.*\))").unwrap();
        if !re.is_match(&code) {
            let result = format!("fun main() {{\r\n{}\r\n}}", code);
            return Some(result);
        }

        None
    }

    fn get_out_path(&self, src_path: &PathBuf) -> PathBuf {
        PathBuf::from(self.get_class_name(src_path))
    }

    fn get_compiler_command(&self, src_path: &PathBuf, _exe_path: &PathBuf) -> Option<String> {
        Some(format!("{} {} -include-runtime -d {}.jar", self.get_compiler(), src_path.to_str().unwrap(), self.get_class_name(src_path)))
    }

    fn get_execution_command(&self, path: &PathBuf) -> String {
        format!("java -jar {}.jar", path.to_str().unwrap())
    }

    fn check_compiler_or_interpreter(&self) -> Expression {
        cmd!(self.get_compiler(), "-version")
    }
}
