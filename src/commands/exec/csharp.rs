use ::Language;
use std::path::PathBuf;
use duct::{ cmd, Expression };

#[derive(Debug)]
pub struct Csharp;

impl Language for Csharp {
    fn get_lang_name(&self) -> String {
        "C#".into()
    }

    fn get_source_file_ext(&self) -> String {
        ".cs".into()
    }

    fn pre_process_code(&self, code: &str) -> Option<String> {
        use regex::Regex;

        let re = Regex::new(r"class\s*.*\s*\{\s*public\s*static\s*void\s*Main\s*\(\s*\)").unwrap();
        if !re.is_match(&code) {
            let result = format!(r"
using System;

public class Program
{{
    public static void Main()
    {{
        {}
    }}
}}", code);
            return Some(result);
        }

        None
    }

    fn get_compiler_command(&self, src_path: PathBuf, exe_path: PathBuf) -> Option<Expression> {
        let compiler;
        let out;
        let target;
        let nologo;
        if cfg!(windows) {
            compiler = "csc";
            out = format!("/out:{}", exe_path.to_str().unwrap());
            target = "/target:winexe";
            nologo = "/nologo";
        } else {
            compiler = "mcs";
            out = format!("-out:{}", exe_path.to_str().unwrap());
            target = "-target:winexe";
            nologo = "-nologo";
        }
        Some(cmd!(compiler, out, target, nologo, src_path))
    }

    fn get_execution_command(&self, path: PathBuf) -> Expression {
        if cfg!(windows) {
            cmd!(path)
        } else {
            cmd!("mono", path)
        }
    }
}
