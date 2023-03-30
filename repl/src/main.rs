use interpreter::{Environment, Interpreter, StdOutPrinter};
use parser::{Parser, Scanner};
use std::io::{BufRead, Write};

fn main() -> anyhow::Result<()> {
    let args = std::env::args().collect::<Vec<String>>();
    if args.len() > 2 {
        eprintln!("Usage: rlox [script]");
        std::process::exit(64);
    } else if args.len() == 2 {
        println!("Reading {}", args[1]);
        run_file(&args[1])?;
    } else {
        run_prompt()?;
    }
    Ok(())
}

fn run_file(path: &str) -> anyhow::Result<()> {
    let source = std::fs::read_to_string(path).unwrap();
    let mut printer = StdOutPrinter;
    let mut interpreter = Interpreter::new(&mut printer);
    run(&source, &mut interpreter)?;
    Ok(())
}

fn run(source: &str, interpreter: &mut Interpreter) -> anyhow::Result<()> {
    let scanner = Scanner::new(source);
    let tokens = scanner.scan_tokens()?;
    let mut parser = Parser::new(tokens);
    let environment = Environment::new_globals_ptr();
    match parser.parse() {
        Ok(statements) => {
            // println!("{:?}", &statements);
            for s in &statements {
                interpreter.evaluate_stmt(&environment, s)?;
            }
        }
        Err(e) => {
            eprintln!("{}", e.to_string());
        }
    }

    Ok(())
}

fn run_prompt() -> anyhow::Result<()> {
    let stdin = std::io::stdin();
    let mut printer = StdOutPrinter;
    let mut interpreter = Interpreter::new(&mut printer);

    loop {
        let mut buf = String::new();

        print!(">>> ");
        std::io::stdout().flush().unwrap();
        match stdin.lock().read_line(&mut buf) {
            Ok(_n) => {
                run(&buf, &mut interpreter)?;
            }
            Err(error) => {
                eprintln!("Error: {error}");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::Parser;
    use crate::scanner::Scanner;

    struct TestPrinter {
        messages: Vec<String>,
    }

    impl TestPrinter {
        fn new() -> Self {
            Self {
                messages: Vec::new(),
            }
        }
    }

    impl Printer for TestPrinter {
        fn print(&mut self, message: &str) {
            self.messages.push(message.to_owned());
        }
    }

    fn print_from(source: &str) -> anyhow::Result<Vec<String>> {
        let mut printer = TestPrinter::new();
        let tokens = Scanner::new(source).scan_tokens()?;
        let mut parser = Parser::new(tokens);
        let statements = parser.parse()?;
        let environment = Environment::new_globals_ptr();
        let mut interpreter = Interpreter::new(&mut printer);
        for s in statements {
            interpreter.evaluate_stmt(&environment, &s)?;
        }
        Ok(printer.messages)
    }

    #[test]
    fn test_source() {
        let source = r"
var a = 1;
var b = 2;
{
    var a = 3;
    var b = 4;
    print a + b;
}
        ";

        assert_eq!(vec!["Number(7.0)"], print_from(source).unwrap());
    }

    #[test]
    fn test_closure() {
        let source = r"
fun counter() {
    var c = 0;
    fun inc() {
        c = c + 1;
        return c;
    }
    return inc;
}
var c = counter();
c();
print c();
";
        assert_eq!(vec!["Number(2.0)"], print_from(source).unwrap());
    }

    #[test]
    fn test_recursion() {
        let source = r"
fun sum(x) {
    if (x < 1) {
        return 0;
    }
    return x + sum(x - 1);
}
print sum(4);
";
        assert_eq!(vec!["Number(10.0)"], print_from(source).unwrap());
    }

    #[test]
    fn test_capturing_using_static_scope() {
        let source = r#"
var a = "global";
{
    fun showA() {
        print a;
    }
    
    showA();
    var a = "block";
    showA();
}
"#;
        assert_eq!(
            vec![r#"String("global")"#, r#"String("global")"#],
            print_from(source).unwrap()
        );
    }
}
