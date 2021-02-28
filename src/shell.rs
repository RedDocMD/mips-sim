use super::sim::*;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::process::exit;

fn help() {
    print!("----------------MIPS ISIM Help------------------------\n");
    print!("go                    - run program to completion     \n");
    print!("run n                 - execute program for n instrs  \n");
    print!("mdump low high        - dump memory from low to high  \n");
    print!("rdump                 - dump the register & bus value \n");
    print!("input reg_num reg_val - set GPR reg_num to reg_val    \n");
    print!("high value            - set the HI register to value  \n");
    print!("low value             - set the LO register to value  \n");
    print!("?                     - display this help menu        \n");
    print!("quit                  - exit the program              \n\n");
}

pub fn prompt(comp: &mut MipsComputer, dump_file: &mut File) -> io::Result<()> {
    print!("MIPS-SIM> ");
    io::stdout().flush()?;
    let mut buf = String::new();
    let bytes = io::stdin().read_line(&mut buf)?;
    if bytes == 0 {
        println!("Bye.");
        exit(0);
    }
    buf = buf.trim_end().to_string();
    println!("");

    let parts: Vec<&str> = buf.split(" ").collect();
    match parts[0] {
        "go" => comp.go(),
        "mdump" => {
            if parts.len() < 3 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "mdump requires 2 params",
                ));
            }
            let start: usize = match parts[1].parse() {
                Ok(val) => val,
                Err(e) => {
                    return Err(io::Error::new(io::ErrorKind::InvalidInput, e));
                }
            };
            let end: usize = match parts[2].parse() {
                Ok(val) => val,
                Err(e) => {
                    return Err(io::Error::new(io::ErrorKind::InvalidInput, e));
                }
            };
            comp.mdump(start, end, dump_file)?;
        }
        "?" => help(),
        "quit" => {
            println!("Bye.");
            exit(0);
        }
        "rdump" => comp.rdump(dump_file)?,
        "run" => {
            if parts.len() < 3 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "run requires 1 param",
                ));
            }
            let cycles: u32 = match parts[1].parse() {
                Ok(val) => val,
                Err(e) => {
                    return Err(io::Error::new(io::ErrorKind::InvalidInput, e));
                }
            };
            comp.run(cycles);
        }
        "input" => {
            if parts.len() < 3 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "input requires 2 params",
                ));
            }
            let register_no: usize = match parts[1].parse() {
                Ok(val) => val,
                Err(e) => {
                    return Err(io::Error::new(io::ErrorKind::InvalidInput, e));
                }
            };
            let register_value: u32 = match parts[2].parse() {
                Ok(val) => val,
                Err(e) => {
                    return Err(io::Error::new(io::ErrorKind::InvalidInput, e));
                }
            };
            comp.curr_state_mut().set_reg(register_no, register_value);
            comp.next_state_mut().set_reg(register_no, register_value);
        }
        "high" => {
            if parts.len() < 3 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "high requires 1 param",
                ));
            }
            let high_reg_val: u32 = match parts[1].parse() {
                Ok(val) => val,
                Err(e) => {
                    return Err(io::Error::new(io::ErrorKind::InvalidInput, e));
                }
            };
            comp.curr_state_mut().set_hi(high_reg_val);
            comp.next_state_mut().set_hi(high_reg_val);
        }
        "low" => {
            if parts.len() < 3 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "low requires 1 param",
                ));
            }
            let low_reg_val: u32 = match parts[1].parse() {
                Ok(val) => val,
                Err(e) => {
                    return Err(io::Error::new(io::ErrorKind::InvalidInput, e));
                }
            };
            comp.curr_state_mut().set_lo(low_reg_val);
            comp.next_state_mut().set_lo(low_reg_val);
        }
        _ => println!("Invalid Command"),
    }
    Ok(())
}
