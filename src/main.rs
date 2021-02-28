use mips_sim::shell::*;
use std::env;
use std::fs::File;
use std::io;
use std::process::exit;

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Usage: {} <program-file-1> <program-file-2> ...", args[0]);
        exit(1);
    }
    println!("MIPS Simulator\n");
    let mut comp = MipsComputer::new(&args[1..])?;
    let mut dump_file = File::create("dumpsim").expect("Can't open dumpsim file");
    loop {
        if let Err(e) = prompt(&mut comp, &mut dump_file) {
            println!("Error: {}", e);
        }
    }
}
