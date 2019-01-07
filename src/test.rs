use LangManager;

#[allow(dead_code)]
fn test_lang(code: &str, lang: &str, ret_code: i32, ignore_compil_stdout: bool, ret_str: &str) {
    let mut lang_manager = LangManager::new();
    let languages = lang_manager.get_languages().clone();
    for (_codes, boxed_lang) in languages {
        lang_manager.set_language_available(boxed_lang.get_lang_name(), true);
    }
    let user = serenity::model::id::UserId::from(123456u64);

    let lang = ::commands::exec::get_lang(&lang_manager, lang).unwrap();
    let res = ::commands::exec::run_code(code.into(), lang, user);
    ::commands::exec::cleanup_user_snippet_directory(user).unwrap();
    let (compilation, execution, _, _) = res.unwrap();

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
    test_lang("print!(\"test\");", "rust", 0, false, "test");
    test_lang("fn main() { print!(\"test\"); }", "rust", 0, false, "test");
}

#[test]
fn test_c() {
    test_lang("printf(\"test\");\nreturn 5;", "c", 5, false, "test");
    test_lang("#include <stdio.h>\n int main() { \nprintf(\"test\");\n return 0;\n }", "c", 0, false, "test");
}

#[test]
fn test_cpp() {
    test_lang("std::cout << \"test\";\nreturn 5;", "cpp", 5, false, "test");
    test_lang("#include <iostream>\n int main() { \nstd::cout << \"test\";\n return 0;\n }", "cpp", 0, false, "test");
}

#[test]
fn test_python() {
    test_lang("print('test', end='')", "python", 0, false, "test");
}

#[test]
fn test_php() {
    test_lang("abc <?php echo \"test\";", "php", 0, false, "abc test");
}

#[test]
fn test_javascript() {
    test_lang("process.stdout.write(\"test\");", "javascript", 0, false, "test");
}

#[test]
fn test_csharp() {
    test_lang("Console.Write(\"test\");", "cs", 0, false, "test");
    test_lang("using System; class Hello { static void Main() { Console.Write(\"test\"); } }", "cs", 0, false, "test");
}

#[test]
fn test_java() {
    test_lang("System.out.print(\"test\");", "java", 0, false, "test");
    test_lang("public class HelloWorld { public static void main(String[] args) { System.out.print(\"test\"); } }", "java", 0, false, "test");
}

#[test]
fn test_lua() {
    test_lang("io.write(\"test\")", "lua", 0, false, "test");
}

#[test]
#[cfg_attr(not(unix), ignore)]
fn test_shell() {
    test_lang("echo \"test\"", "shell", 0, false, "test\n");
}

#[test]
#[cfg_attr(not(unix), ignore)]
fn test_asmx64() {
    let code = r#"section .text

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
syscall"#;
    test_lang(code, "asmx64", 0, false, "8");
}

#[test]
#[cfg_attr(not(unix), ignore)]
fn test_asmx86() {
    let code = r#"section .text

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
int 80h"#;
    test_lang(code, "asmx86", 0, false, "8");
}

#[test]
fn test_vb() {
    test_lang("Console.Write(\"test\")", "vb", 0, true, "test");
    test_lang("Imports System\r\nModule HelloWorld\r\nSub Main()\r\nConsole.Write(\"test\")\r\nEnd Sub\r\nEnd Module", "vb", 0, true, "test");
}

#[test]
fn test_kotlin() {
    test_lang("print(\"test\")", "kt", 0, false, "test");
    test_lang("fun main(args: Array<String>) {\nprint(\"test\")\n}", "kt", 0, false, "test");
}

#[test]
#[cfg(ignore)]
fn test_env() {
    assert_eq!(true, ::is_running_as_docker_container());
}
