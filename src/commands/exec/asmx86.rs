use std::path::PathBuf;

use crate::commands::exec::language::Language;

#[derive(Debug)]
pub struct Asmx86;

impl Asmx86 {
    fn get_file_name(&self, src_path: &PathBuf) -> String {
        src_path
            .with_extension("")
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .into()
    }
}

impl Language for Asmx86 {
    fn get_image_name(&self) -> String {
        "rustacean-asm32".into()
    }

    fn get_lang_name(&self) -> String {
        "Asm32".into()
    }

    fn get_source_file_ext(&self) -> String {
        ".asm".into()
    }

    fn get_compiler_command(&self, src_path: &PathBuf, exe_path: &PathBuf) -> Option<String> {
        Some(format!(
            "nasm -f elf32 {} && ld -melf_i386 {}.o -o {}",
            src_path.to_str().unwrap(),
            self.get_file_name(src_path),
            exe_path.to_str().unwrap()
        ))
    }

    fn check_compiler_or_interpreter(&self) -> String {
        String::from("nasm -version && ld -version")
    }
}
