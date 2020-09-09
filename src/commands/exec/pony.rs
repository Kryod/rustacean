use std::path::PathBuf;

use crate::commands::exec::language::Language;

#[derive(Debug)]
pub struct Pony;

impl Language for Pony {
    fn get_image_name(&self) -> String {
        "rustacean-pony".into()
    }

    fn get_lang_name(&self) -> String {
        "Pony".into()
    }

    fn get_source_file_ext(&self) -> String {
        ".pony".into()
    }

    fn pre_process_code(&self, code: &str, _src_path: &PathBuf) -> Option<String> {
        use regex::Regex;

        let re = Regex::new(r"(?s)(actor\s+Main\s*\n+^new create\(.*\)\s=>)/m").unwrap();
        if !re.is_match(&code) {
            // let result = format!("use \"stdlib\"\r\nuse \"random\"\r\nuse \"serialize\"\r\n use \"options\"\r\n use \"file\"\r\nuse \" builtin\"\r\nuse \"base64\"\r\nuse \"json\"\r\nuse \"itertools\"\r\nuse \"logger\"\r\n use\"math\"\r\nuse \"format\"\r\nactor Main\r\nnew create(env: Env)=>\r\n{}", code);
		let result = format!(" use \"random\"\r\n use \"json\" \r\n use \"itertools\"\r\n actor Main\r\nnew create(env: Env)=>\r\n{}",code);
            return Some(result);
        }

        None
    }
    fn get_execution_command(&self, path:&PathBuf) -> String {
		String::from(path.to_str().unwrap())
	}

    fn get_compiler_command(&self, src_path: &PathBuf, dest_path : &PathBuf) -> Option<String> {
        let c;
        match src_path.is_dir() {
            false => {
                c=src_path.parent().unwrap();
            }
            true => {
                c=src_path; 
            }
        }
	println!("{}", c.to_str().unwrap());
        Some(format!(
            "ponyc {} ",
                c.to_str().unwrap()
            ))
   }
    fn get_out_path(&self, src_path: &PathBuf) -> PathBuf {
	let mut t = src_path.to_path_buf();
	t.pop();
	//println!("{0}/pony",t.to_str().unwrap());

	PathBuf::from(format!("{0}/{0}",t.to_str().unwrap()))
    }

    fn check_compiler_or_interpreter(&self) -> String {
        String::from("ponyc --version")
    }
}
