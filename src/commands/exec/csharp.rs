use std::path::PathBuf;

use crate::commands::exec::language::Language;

#[derive(Debug)]
pub struct Csharp;

impl Language for Csharp {
    fn get_image_name(&self) -> String {
        "rustacean-csharp".into()
    }

    fn get_lang_name(&self) -> String {
        "CSharp".into()
    }

    fn get_source_file_ext(&self) -> String {
        ".cs".into()
    }

    fn pre_process_code(&self, code: &str, _src_path: &PathBuf) -> Option<String> {
        use regex::Regex;

        let re = Regex::new(r"(?s)(class\s+.*\{.*static\s+void\s+Main\s*\(.*\))").unwrap();
        if !re.is_match(&code) {
            let result = format!(
                r"
using System;

public class Program
{{
    public static void Main()
    {{
        {}
    }}
}}",
                code
            );
            return Some(result);
        }

        None
    }

    fn get_compiler_command(&self, src_path: &PathBuf, exe_path: &PathBuf) -> Option<String> {
        let (compiler, out, target, nologo) = if cfg!(windows) {
            (
                "csc",
                format!("/out:{}", exe_path.to_str().unwrap()),
                "/target:winexe",
                "/nologo",
            )
        } else {
            (
                "mcs",
                format!("-out:{}", exe_path.to_str().unwrap()),
                "-target:winexe",
                "-nologo",
            )
        };
        Some(format!(
            "{} {} {} {} {}",
            compiler,
            out,
            target,
            nologo,
            src_path.to_str().unwrap()
        ))
    }

    fn get_execution_command(&self, path: &PathBuf) -> String {
        if cfg!(windows) {
            String::from(path.to_str().unwrap())
        } else {
            format!("mono {}", path.to_str().unwrap())
        }
    }

    fn check_compiler_or_interpreter(&self) -> String {
        if cfg!(windows) {
            String::from("csc /version")
        } else {
            String::from("mcs --version")
        }
    }
}
