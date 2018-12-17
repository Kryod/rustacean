use std::path::PathBuf;

use duct::{ cmd, Expression };

pub trait Language {
    fn get_lang_name(&self) -> String;
    fn get_source_file_ext(&self) -> String;
    fn get_out_path(&self, src_path: &PathBuf) -> PathBuf {
        let path = format!("{}.out", src_path.to_str().unwrap());
        PathBuf::from(path)
    }
    fn pre_process_code(&self, _code: &str, _src_path: &PathBuf) -> Option<String> {
        None
    }
    fn get_compiler_command(&self, src_path: &PathBuf, exe_path: &PathBuf) -> Option<Expression> {
        let _ = std::fs::copy(src_path, exe_path);
        None
    }
    fn get_execution_command(&self, path: &PathBuf) -> Expression {
        cmd!(path)
    }
}
