mod ast;
mod interpreter;
mod parser;
mod scanner;
mod token;
mod value;

use crate::ast::DepthPrinter;
use crate::interpreter::Interpreter;
use crate::parser::Parser;
use crate::scanner::Scanner;
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
    run(&source)?;
    Ok(())
}

fn run(source: &str) -> anyhow::Result<()> {
    let scanner = Scanner::new(source);
    let tokens = scanner.scan_tokens()?;

    let mut parser = Parser::new(tokens);
    match parser.parse() {
        Ok(expr) => {
            let mut printer = DepthPrinter::new();
            printer.visit(&expr);
            printer.print();
            println!("{}", "=".repeat(10));
            let mut interpreter = Interpreter::new();
            println!("{:?}", interpreter.evaluate_expr(&expr));
        }
        Err(e) => {
            eprintln!("{}", e.to_string());
        }
    }

    Ok(())
}

fn run_prompt() -> anyhow::Result<()> {
    let stdin = std::io::stdin();

    loop {
        let mut buf = String::new();

        print!(">>> ");
        std::io::stdout().flush().unwrap();
        match stdin.lock().read_line(&mut buf) {
            Ok(_n) => {
                run(&buf)?;
            }
            Err(error) => {
                eprintln!("Error: {error}");
            }
        }
    }
}
