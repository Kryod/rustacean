use LangManager;
use ::commands;

use std::path::PathBuf;
use std::fs;

fn get_test_user() -> serenity::model::user::User {
    serenity::model::user::User {
        avatar: None,
        bot: false,
        discriminator: 0,
        id: serenity::model::id::UserId::from(123456u64),
        name: String::from("test")
    }
}

fn cleanup(src_path: &PathBuf, exe_path: Option<&PathBuf>) {
    let _ = std::fs::remove_file(src_path);
    if let Some(exe_path) = exe_path {
        let _ = std::fs::remove_file(exe_path);
    };
}

#[allow(dead_code)]
fn test_lang(code: String, lang: String, ret_code: i32, ignore_compil_stdout: bool, ret_str: String) {
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
            Err(e) => {
                cleanup(&src_path, None);
                panic!("Could not save code snippet: {}", e);
            },
        };
    }

    let out_path = lang.get_out_path(&src_path);
    let compilation = match lang.get_compiler_command(&src_path, &out_path) {
        Some(command) => commands::exec::run_command(&src_path, command, 20),
        None => Ok(commands::exec::CommandResult::default())
    };
    let compilation = match compilation {
        Ok(res) => res,
        Err(e) => {
            cleanup(&src_path, None);
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
            match commands::exec::run_command(&src_path, lang.get_execution_command(&out_path), 20) {
                Ok(res) => res,
                Err(e) => {
                    cleanup(&src_path, Some(&out_path));
                    panic!("An error occurred while running code snippet: {}", e);
                }
            }
        }
    };

    cleanup(&src_path, Some(&out_path));

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
                if !ignore_compil_stdout && !compilation.stdout.is_empty() {
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
    test_lang(String::from("print!(\"test\");"), String::from("rust"), 0, false, String::from("test"));
    test_lang(String::from("fn main() { print!(\"test\"); }"), String::from("rust"), 0, false, String::from("test"));
}

#[test]
fn test_c() {
    test_lang(String::from("printf(\"test\");\nreturn 5;"), String::from("c"), 5, false, String::from("test"));
    test_lang(String::from("#include <stdio.h>\n int main() { \nprintf(\"test\");\n return 0;\n }"), String::from("c"), 0, false, String::from("test"));
}

#[test]
fn test_cpp() {
    test_lang(String::from("std::cout << \"test\";\nreturn 5;"), String::from("cpp"), 5, false, String::from("test"));
    test_lang(String::from("#include <iostream>\n int main() { \nstd::cout << \"test\";\n return 0;\n }"), String::from("cpp"), 0, false, String::from("test"));
}

#[test]
fn test_python() {
    test_lang(String::from("print('test', end='')"), String::from("python"), 0, false, String::from("test"));
}

#[test]
fn test_php() {
    test_lang(String::from("abc <?php echo \"test\";"), String::from("php"), 0, false, String::from("abc test"));
}

#[test]
fn test_javascript() {
    test_lang(String::from("process.stdout.write(\"test\");"), String::from("javascript"), 0, false, String::from("test"));
}

#[test]
fn test_csharp() {
    test_lang(String::from("Console.Write(\"test\");"), String::from("cs"), 0, false, String::from("test"));
    test_lang(String::from("using System; class Hello { static void Main() { Console.Write(\"test\"); } }"), String::from("cs"), 0, false, String::from("test"));
}

#[test]
fn test_java() {
    test_lang(String::from("System.out.print(\"test\");"), String::from("java"), 0, false, String::from("test"));
    test_lang(String::from("public class HelloWorld { public static void main(String[] args) { System.out.print(\"test\"); } }"), String::from("java"), 0, false, String::from("test"));
}

#[test]
fn test_lua() {
    test_lang(String::from("io.write(\"test\")"), String::from("lua"), 0, false, String::from("test"));
}

#[test]
#[cfg_attr(not(unix), ignore)]
fn test_shell() {
    test_lang(String::from("echo \"test\""), String::from("shell"), 0, false, String::from("test\n"));
}

#[test]
#[cfg_attr(not(unix), ignore)]
fn test_asmx64() {
    let code = String::from(r#"section .text

    global _start

_start:
mov rax, 0x38
push rax

mov rax, 1
mov rdi, 1
mov rsi, rsp
mov rdx, 1
syscall

mov rax, 60
xor rdi, rdi
syscall"#);
    test_lang(code, String::from("asmx64"), 0, false, String::from("8"));
}

#[test]
#[cfg_attr(not(unix), ignore)]
fn test_asmx86() {
    let code = String::from(r#"section .text

    global _start

_start:
mov eax, 0x38
push eax

mov eax, 4
mov ebx, 1
mov ecx, esp
mov edx, 1
int 80h

mov eax, 1
xor ebx, ebx
int 80h"#);
    test_lang(code, String::from("asmx86"), 0, false, String::from("8"));
}

#[test]
fn test_vb() {
    test_lang(String::from("Console.Write(\"test\")"), String::from("vb"), 0, true, String::from("test"));
    test_lang(String::from("Imports System\r\nModule HelloWorld\r\nSub Main()\r\nConsole.Write(\"test\")\r\nEnd Sub\r\nEnd Module"), String::from("vb"), 0, true, String::from("test"));
}

#[test]
fn test_kotlin() {
    test_lang(String::from("print(\"test\")"), String::from("kt"), 0, false, String::from("test"));
    test_lang(String::from("fun main(args: Array<String>) {\nprint(\"test\")\n}"), String::from("kt"), 0, false, String::from("test"));
}
