#[macro_use]
extern crate num_derive;
extern crate num_traits;

mod chunk;
mod common;
mod compiler;
mod object;
mod scanner;
mod value;
mod vm;
use std::env;
use std::io::Write;
use std::process::exit;
use vm::InterpretResult;
use vm::VM;

fn main() {
    env::set_var("RUST_BACKTRACE", "1");
    let args: Vec<String> = env::args().collect();
    let mut vm = VM::new();

    if args.len() == 1 {
        repl(&mut vm);
    } else if args.len() == 2 {
        run_file(&mut vm, &args[1]);
    } else {
        println!("Usage: rlox [script]");
        exit(64);
    }
}

fn repl(vm: &mut VM) {
    println!("Welcome to rMAX!");
    loop {
        print!("MAX > ");
        std::io::stdout().flush().unwrap();

        let mut line = String::new();
        std::io::stdin().read_line(&mut line).unwrap();

        if line.is_empty() {
            break;
        }

        vm.interpret(line);
    }
}

fn run_file(vm: &mut VM, file: &str) {
    let source = std::fs::read_to_string(file).unwrap();
    let result = vm.interpret(source);

    match result {
        InterpretResult::Ok => (),
        InterpretResult::CompileError => exit(65),
        InterpretResult::RuntimeError => exit(70),
    }
}
