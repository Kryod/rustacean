use std::path::PathBuf;

use commands::exec::language::Language;
use duct::{ cmd, Expression };

#[derive(Debug)]
pub struct Asm;

impl Asm {
    fn get_file_name(&self, src_path: &PathBuf) -> String {
        src_path.with_extension("").file_name().unwrap().to_str().unwrap().into()
    }
}

impl Language for Asm {
    fn get_lang_name(&self) -> String {
        "Asm".into()
    }

    fn get_source_file_ext(&self) -> String {
        ".asm".into()
    }

    fn get_compiler_command(&self, src_path: &PathBuf, exe_path: &PathBuf) -> Option<Expression> {
        info!("nasm -f elf64 {} ld {} -o {}", src_path.to_str().unwrap(), format!("{}.o", self.get_file_name(src_path)), exe_path.to_str().unwrap());
        Some(cmd!("nasm", "-f", "elf64", src_path)
            .then(cmd!("ld", format!("{}.o", self.get_file_name(src_path)),"-o", exe_path)))
    }
}
