use std::path::PathBuf;

use crate::commands::exec::language::Language;

#[derive(Debug)]
pub struct OCaml;

impl Language for OCaml {
  fn get_image_name(&self) -> String {
    "rustacean-ocaml".into()
  }

  fn get_lang_name(&self) -> String {
    "OCaml".into()
  }

  fn get_source_file_ext(&self) -> String {
    ".ml".into()
  }

  fn get_execution_command(&self, path: &PathBuf) -> String {
    format!("ocaml {}", path.to_str().unwrap())
  }

  fn check_compiler_or_interpreter(&self) -> String {
    "ocaml --version".into()
  }
}
