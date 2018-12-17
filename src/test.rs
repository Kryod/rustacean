use LangManager;
use std::fs;

use ::commands;

fn get_test_user() -> serenity::model::user::User {
    serenity::model::user::User {
        avatar: None,
        bot: false,
        discriminator: 0,
        id: serenity::model::id::UserId::from(123456u64),
        name: String::from("test")
    }
}

#[allow(dead_code)]
fn test_lang(code: String, lang: String, ret_code: i32, ret_str: String) {
    let lang_manager = LangManager::new();
    let code = code;
    let lang = match LangManager::get(&lang_manager, &lang) {
        Some(lang) => lang,
        None => {
            let langs = lang_manager.get_languages_list();
            panic!("Unknown programming language\nHere are the languages available: {}", langs);
        }
    };

    let user = get_test_user();
    let src_path = commands::exec::save_code(&code, &user, &lang.get_source_file_ext()).unwrap();

    if let Some(modified) = lang.pre_process_code(&code, &src_path) {
        match fs::write(src_path.as_path(), modified) {
            Ok(_) => {},
            Err(e) => panic!("Could not save code snippet: {}", e),
        };
    }

    let out_path = lang.get_out_path(&src_path);
    let compilation = match lang.get_compiler_command(&src_path, &out_path) {
        Some(command) => commands::exec::run_command(&src_path, command),
        None => Ok(commands::exec::CommandResult::default())
    };
    let compilation = match compilation {
        Ok(res) => res,
        Err(e) => {
            panic!("An error occurred while compiling code snippet: {}", e);
        },
    };

    let execution = match compilation.exit_code {
        Some(code) if code != 0 => {
            // Return a default value if compilation failed
            commands::exec::CommandResult::default()
        },
        _ => {
            // Compilation succeeded, run the snippet
            match commands::exec::run_command(&src_path, lang.get_execution_command(&out_path)) {
                Ok(res) => res,
                Err(e) => {
                    panic!("An error occurred while running code snippet: {}", e);
                }
            }
        }
    };

    if compilation.timed_out {
        // Compilation timed out
        panic!("Compilation timed out");
    } else if execution.timed_out {
        // Execution timed out
        panic!("Execution timed out");
    } else {
        // Didn't time out
        match compilation.exit_code {
            Some(code) if code != 0 => {
                // Compilation failed
                panic!("Compilation failed: ```\r\n{}```",compilation.stderr);
            },
            _ => {
                // Compilation succeeded
                if !compilation.stdout.is_empty() {
                    panic!("Compilation output: ```\r\n{}```", compilation.stdout);
                }
                if !compilation.stderr.is_empty() {
                    panic!("Compilation error output: ```\r\n{}```", compilation.stderr);
                }
                if let Some(code) = execution.exit_code {
                    assert_eq!(code, ret_code);
                }
                if !execution.stdout.is_empty() {
                    assert_eq!(execution.stdout, ret_str);
                }
                if !execution.stderr.is_empty() {
                    panic!("Error output: ```\r\n{}```", execution.stderr);
                }
            }
        };
    }
}

#[test]
fn test_rust() {
    test_lang(String::from("print!(\"test\");"), String::from("rust"), 0, String::from("test"));
    test_lang(String::from("fn main() { print!(\"test\"); }"), String::from("rust"), 0, String::from("test"));
}

#[test]
fn test_c() {
    test_lang(String::from("printf(\"test\");\nreturn 5;"), String::from("c"), 5, String::from("test"));
    test_lang(String::from("#include <stdio.h>\n int main() { \nprintf(\"test\");\n return 0;\n }"), String::from("c"), 0, String::from("test"));
}

#[test]
fn test_cpp() {
    test_lang(String::from("std::cout << \"test\";\nreturn 5;"), String::from("cpp"), 5, String::from("test"));
    test_lang(String::from("#include <iostream>\n int main() { \nstd::cout << \"test\";\n return 0;\n }"), String::from("cpp"), 0, String::from("test"));
}

#[test]
fn test_python() {
    test_lang(String::from("print(\"test\", end=\"\")"), String::from("python"), 0, String::from("test"));
}

#[test]
fn test_php() {
    test_lang(String::from("abc <?php echo \"test\";"), String::from("php"), 0, String::from("abc test"));
}

#[test]
fn test_javascript() {
    test_lang(String::from("process.stdout.write(\"test\");"), String::from("javascript"), 0, String::from("test"));
}

#[test]
fn test_csharp() {
    test_lang(String::from("Console.Write(\"test\");"), String::from("cs"), 0, String::from("test"));
}

#[test]
fn test_java() {
    test_lang(String::from("System.out.print(\"test\");"), String::from("java"), 0, String::from("test"));
}

#[test]
fn test_lua() {
    test_lang(String::from("io.write(\"test\")"), String::from("lua"), 0, String::from("test"));
}

#[test]
#[cfg_attr(not(unix), ignore)]
fn test_shell() {
    test_lang(String::from("echo \"test\""), String::from("shell"), 0, String::from("test\n"));
}
