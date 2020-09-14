use std::path::PathBuf;

use crate::commands::exec::language::Language;

#[derive(Debug)]
pub struct Prolog;

impl Language for Prolog {
    fn get_image_name(&self) -> String {
        "rustacean-prolog".into()
    }

    fn get_lang_name(&self) -> String {
        "Prolog".into()
    }

    fn get_source_file_ext(&self) -> String {
        ".pl".into()
    }

    fn get_execution_command(&self, path: &PathBuf) -> String {
        format!("swipl -g true <{}", path.to_str().unwrap())
    }

    fn check_compiler_or_interpreter(&self) -> String {
        String::from("prolog --version")
    }

    fn pre_process_code(&self, code: &str, _src_path: &PathBuf) -> Option<String> {

        let re = Regex::new(r"(.|\n)*halt.\m").unwrap();
        if !re.is_match(&code) {
            let result = format!("{}", code);
            return Some(result);
        } else {
            let result = format!("{}\nhalt.", code);
            return Some(result);
        }
    }
}
