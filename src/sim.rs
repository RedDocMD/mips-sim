use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::path::Path;

pub const MIPS_REGS: usize = 32;

#[derive(Clone, Copy)]
pub struct CpuState {
    pc: u32,
    regs: [u32; MIPS_REGS],
    hi: u32,
    lo: u32,
}

struct MemRegion {
    start: usize,
    size: usize,
    mem: Vec<u8>,
}

pub struct MipsComputer {
    curr_state: CpuState,
    next_state: CpuState,
    run_bit: bool,
    instr_cnt: u32,
    memory: [MemRegion; 5],
}

impl CpuState {
    fn new() -> Self {
        Self {
            pc: 0,
            regs: [0; MIPS_REGS],
            hi: 0,
            lo: 0,
        }
    }

    pub fn set_hi(&mut self, val: u32) {
        self.hi = val;
    }

    pub fn set_lo(&mut self, val: u32) {
        self.lo = val;
    }

    pub fn set_reg(&mut self, reg: usize, val: u32) -> bool {
        if reg < MIPS_REGS {
            self.regs[reg] = val;
            true
        } else {
            false
        }
    }
}

impl MemRegion {
    fn new(start: usize, size: usize) -> Self {
        Self {
            start,
            size,
            mem: vec![0; size],
        }
    }

    fn contains_address(&self, address: usize) -> bool {
        address >= self.start && address < (self.start + self.size)
    }

    fn read_32(&self, address: usize) -> Option<u32> {
        if !self.contains_address(address) {
            return None;
        } else {
            let offset = address - self.start;
            let byte3 = self.mem[offset + 3] as u32;
            let byte2 = self.mem[offset + 2] as u32;
            let byte1 = self.mem[offset + 1] as u32;
            let byte0 = self.mem[offset] as u32;
            return Some((byte3 << 24) | (byte2 << 16) | (byte1 << 8) | byte0);
        }
    }

    fn write_32(&mut self, address: usize, value: u32) -> bool {
        if !self.contains_address(address) {
            return false;
        } else {
            let offset = address - self.start;
            self.mem[offset + 3] = (value >> 24) as u8;
            self.mem[offset + 2] = (value >> 16) as u8;
            self.mem[offset + 1] = (value >> 8) as u8;
            self.mem[offset] = value as u8;
            return true;
        }
    }

    // Bytes must be in little-endian order (LSB at lowest address)
    fn write_bytes(&mut self, address: usize, bytes: &[u8]) -> bool {
        if !self.contains_address(address) {
            return false;
        } else {
            let offset = address - self.start;
            for (idx, byte) in bytes.iter().enumerate() {
                self.mem[offset + idx] = *byte;
            }
            return true;
        }
    }
}

pub const MEM_DATA_START: usize = 0x10000000;
pub const MEM_DATA_SIZE: usize = 0x00100000;
pub const MEM_TEXT_START: usize = 0x00400000;
pub const MEM_TEXT_SIZE: usize = 0x00100000;
pub const MEM_STACK_START: usize = 0x7ff00000;
pub const MEM_STACK_SIZE: usize = 0x00100000;
pub const MEM_KDATA_START: usize = 0x90000000;
pub const MEM_KDATA_SIZE: usize = 0x00100000;
pub const MEM_KTEXT_START: usize = 0x80000000;
pub const MEM_KTEXT_SIZE: usize = 0x00100000;

impl MipsComputer {
    pub fn new(filenames: &[String]) -> io::Result<Self> {
        let mut comp = Self {
            curr_state: CpuState::new(),
            next_state: CpuState::new(),
            run_bit: true,
            instr_cnt: 0,
            memory: [
                MemRegion::new(MEM_DATA_START, MEM_DATA_SIZE),
                MemRegion::new(MEM_TEXT_START, MEM_TEXT_SIZE),
                MemRegion::new(MEM_STACK_START, MEM_STACK_SIZE),
                MemRegion::new(MEM_KDATA_START, MEM_KDATA_SIZE),
                MemRegion::new(MEM_KTEXT_START, MEM_KTEXT_SIZE),
            ],
        };
        for filename in filenames.iter() {
            comp.load_program(filename)?;
        }
        comp.next_state = comp.curr_state;
        Ok(comp)
    }

    fn load_program<T: AsRef<Path>>(&mut self, path: T) -> io::Result<()> {
        let mut file = File::open(&path).expect(&format!(
            "Cannot open program file {}",
            path.as_ref().display()
        ));
        let mut buf = [0 as u8; 4];
        let mut off = 0;
        loop {
            buf.fill(0);
            let bytes_read = file.read(&mut buf)?;
            if bytes_read == 0 {
                // EOF
                break;
            }
            if bytes_read < 4 {
                buf.rotate_left(bytes_read);
            }
            buf.reverse(); // Big-endian to little-endian
            self.mem_write_bytes(MEM_TEXT_START + off, &buf);
            off += 4;
        }
        self.curr_state.pc = MEM_TEXT_START as u32;
        println!("Read {} words from program into memory.\n", off / 4);
        Ok(())
    }

    fn mem_read_32(&self, address: usize) -> Option<u32> {
        for mem_reg in &self.memory {
            if let Some(data) = mem_reg.read_32(address) {
                return Some(data);
            }
        }
        return None;
    }

    fn mem_write_32(&mut self, address: usize, value: u32) -> bool {
        for mem_reg in &mut self.memory {
            if mem_reg.write_32(address, value) {
                return true;
            }
        }
        return false;
    }

    fn mem_write_bytes(&mut self, address: usize, bytes: &[u8]) -> bool {
        for mem_reg in &mut self.memory {
            if mem_reg.write_bytes(address, bytes) {
                return true;
            }
        }
        return false;
    }

    fn process_instruction(&mut self) {
        unimplemented!("Cannot yet process instruction!")
    }

    pub fn cycle(&mut self) {
        self.process_instruction();
        self.curr_state = self.next_state;
        self.instr_cnt += 1;
    }

    pub fn run(&mut self, num_cycles: u32) {
        if !self.run_bit {
            println!("Can't simulate, Simulator halted\n");
        } else {
            println!("Simulating for {} cycles...\n", num_cycles);
            for _i in 0..num_cycles {
                if !self.run_bit {
                    println!("Simulator halted\n");
                    break;
                }
                self.cycle();
            }
        }
    }

    pub fn go(&mut self) {
        if !self.run_bit {
            println!("Can't simulate, Simulator halted\n");
        } else {
            println!("Simulating...\n");
            while self.run_bit {
                self.cycle();
            }
            println!("Simulator halted\n");
        }
    }

    fn mdump_intern<T: Write>(&self, start: usize, stop: usize, out: &mut T) -> io::Result<()> {
        let mut address: usize;

        writeln!(out, "\nMemory content [{:#08X}..{:#08X}] :", start, stop)?;
        writeln!(out, "-----------------------------------------")?;
        address = start;
        while address <= stop {
            if let Some(value) = self.mem_read_32(address) {
                writeln!(out, "    {:#08X}  ({}) : {:#08X}", address, address, value)?;
            } else {
                writeln!(
                    out,
                    "    {:#08X}  ({}) : <undefined address>",
                    address, address
                )?;
            }
            address += 4;
        }
        writeln!(out, "")?;

        Ok(())
    }

    pub fn mdump(&self, start: usize, stop: usize, file: &mut File) -> io::Result<()> {
        self.mdump_intern(start, stop, &mut io::stdout())?;
        self.mdump_intern(start, stop, file)?;
        Ok(())
    }

    fn rdump_intern<T: Write>(&self, out: &mut T) -> io::Result<()> {
        writeln!(out, "\n Current reigster/bus values :")?;
        writeln!(out, "-------------------------------")?;
        writeln!(out, "Instruction count : {}", self.instr_cnt)?;
        writeln!(out, "PC                : {:#08X}", self.curr_state.pc)?;
        writeln!(out, "Registers:")?;
        for (i, reg) in self.curr_state.regs.iter().enumerate() {
            writeln!(out, "R{}: {:#08X}", i, reg)?;
        }
        writeln!(out, "HI: {:#08X}", self.curr_state.hi)?;
        writeln!(out, "LO: {:#08X}", self.curr_state.lo)?;
        writeln!(out, "")?;
        Ok(())
    }

    pub fn rdump(&self, file: &mut File) -> io::Result<()> {
        self.rdump_intern(&mut io::stdout())?;
        self.rdump_intern(file)?;
        Ok(())
    }

    pub fn curr_state(&self) -> &CpuState {
        &self.curr_state
    }

    pub fn curr_state_mut(&mut self) -> &mut CpuState {
        &mut self.curr_state
    }

    pub fn next_state(&self) -> &CpuState {
        &self.next_state
    }

    pub fn next_state_mut(&mut self) -> &mut CpuState {
        &mut self.next_state
    }
}
