use std::env::args;
use std::io::{Result, BufRead, Write, stdin, stdout};


mod vm;
use vm::*;
mod chunks;
mod token;
mod token_type;
mod value;
mod compliler;
mod scanner;

fn main() {
    let args: Vec<String> = args().collect();
     let mut vm = VM::new();  

    match args.len() {
        1 => repl(&mut vm),
        2 => run_file(&mut vm, &args[1]).expect("Could not run the file"),
        _ => {
            println!("Usage: lox-bytecode [path]");
            std::process::exit(64);
        }
    }     
   
}

fn repl(vm: &mut VM) {
    let stdin = stdin();
    print!("> ");
    let _ = stdout().flush();
    for line in stdin.lock().lines() {
        if let Ok(line) = line {
            if line.is_empty() {
                break;
            }
            let _ = vm.interpret(&line);
        } else {
            break;
        }
        print!("> ");
        let _ = stdout().flush();
    }

}

fn run_file(vm: &mut VM, path: &str) -> Result<()>{
    let  buf = std::fs::read_to_string(path)?;
    match vm.interpret(&buf) {
        Err(InterpretResult::CompileError) => std::process::exit(65),
        Err(InterpretResult::RuntimeError) => std::process::exit(70),
        _ =>  std::process::exit(0),
    }    
}