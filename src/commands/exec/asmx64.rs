use std::path::PathBuf;

use commands::exec::language::Language;
use duct::{ cmd, Expression };

#[derive(Debug)]
pub struct Asmx64;

impl Asmx64 {
    fn get_file_name(&self, src_path: &PathBuf) -> String {
        src_path.with_extension("").file_name().unwrap().to_str().unwrap().into()
    }
}

impl Language for Asmx64 {
    
    fn get_image_name(&self) -> String {
        "rustacean-asm64".into()
    }

    fn get_lang_name(&self) -> String {
        "Asm64".into()
    }

    fn get_source_file_ext(&self) -> String {
        ".asm".into()
    }

    fn get_compiler_command(&self, src_path: &PathBuf, exe_path: &PathBuf) -> Option<String> {

        Some(format!("nasm -f elf32 {} && ld {}.o -o {}", src_path.to_str().unwrap(), self.get_file_name(src_path), exe_path.to_str().unwrap()))
        //Some(cmd!("nasm", "-f", "elf64", src_path)
        //    .then(cmd!("ld", format!("{}.o", self.get_file_name(src_path)), "-o", exe_path)))
    }

    fn check_compiler_or_interpreter(&self) -> Expression {
        cmd!("nasm", "-version").then(cmd!("ld", "-version"))
    }
}
