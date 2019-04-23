use std::path::PathBuf;

use commands::exec::language::Language;
use duct::{ cmd, Expression };

#[derive(Debug)]
pub struct Java;

impl Java {
    fn get_class_name(&self, src_path: &PathBuf) -> String {
        src_path.with_extension("").file_name().unwrap().to_str().unwrap().into()
    }
}

impl Language for Java {
    fn get_image_name(&self) -> String {
        "gcc".into()
    }

    fn get_lang_name(&self) -> String {
        "Java".into()
    }

    fn get_source_file_ext(&self) -> String {
        ".java".into()
    }

    fn pre_process_code(&self, code: &str, src_path: &PathBuf) -> Option<String> {
        let class_name = self.get_class_name(src_path);

        use regex::Regex;

        let re = Regex::new(r"(?P<start>.*class\s*)(?P<name>.*?)(?P<end>\s*\{\s*public\s*static\s*void\s*main\s*\(.*\).*)").unwrap();
        if !re.is_match(&code) {
            let result = format!(r"
public class {} {{
    public static void main(String[] args) {{
        {}
    }}
}}", class_name, code);
            return Some(result);
        } else {
            return Some(re.replace(code, format!("$start {} $end", class_name).as_str()).into());
        }
    }

    fn get_out_path(&self, src_path: &PathBuf) -> PathBuf {
        PathBuf::from(self.get_class_name(src_path))
    }

    fn get_compiler_command(&self, src_path: &PathBuf, _exe_path: &PathBuf) -> Option<String> {
        Some(format!("javac {}", src_path.to_str().unwrap()))
    }

    fn get_execution_command(&self, path: &PathBuf) -> String {
        format!("java {}", path.to_str().unwrap())
    }

    fn check_compiler_or_interpreter(&self) -> Expression {
        cmd!("javac", "-version")
    }
}
