use rlox_interpreter::{Environment, Interpreter, Printer};
use rlox_parser::{Parser, Scanner};

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
#[ignore]
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
