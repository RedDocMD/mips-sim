use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::path::Path;

use super::instr::*;

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
            None
        } else {
            let offset = address - self.start;
            let byte3 = self.mem[offset + 3] as u32;
            let byte2 = self.mem[offset + 2] as u32;
            let byte1 = self.mem[offset + 1] as u32;
            let byte0 = self.mem[offset] as u32;
            Some((byte3 << 24) | (byte2 << 16) | (byte1 << 8) | byte0)
        }
    }

    fn read_8(&self, address: usize) -> Option<u8> {
        if !self.contains_address(address) {
            None
        } else {
            let offset = address - self.start;
            Some(self.mem[offset])
        }
    }

    fn read_16(&self, address: usize) -> Option<u16> {
        if !self.contains_address(address) {
            None
        } else {
            let offset = address - self.start;
            let byte1 = self.mem[offset + 1] as u16;
            let byte0 = self.mem[offset] as u16;
            Some((byte1 << 8) | byte0)
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

    fn mem_read_16(&self, address: usize) -> Option<u16> {
        for mem_reg in &self.memory {
            if let Some(data) = mem_reg.read_16(address) {
                return Some(data);
            }
        }
        return None;
    }

    fn mem_read_8(&self, address: usize) -> Option<u8> {
        for mem_reg in &self.memory {
            if let Some(data) = mem_reg.read_8(address) {
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
        let instr = self.mem_read_32(self.curr_state.pc as usize);
        if let Some(instr) = instr {
            if instr == 0 {
                self.run_bit = false;
            } else {
                let instr = parse_instr(instr);
                println!("Processing {:?}", instr);
                let incr_pc = match instr {
                    Instr::JType(instr) => self.process_jtype_instruction(&instr),
                    Instr::IType(instr) => self.process_itype_instruction(&instr),
                    Instr::RType(instr) => self.process_rtype_instruction(&instr),
                };
                if incr_pc {
                    self.next_state.pc = self.curr_state.pc + 4;
                }
            }
        } else {
            self.run_bit = false;
        }
    }

    fn process_jtype_instruction(&mut self, instr: &JType) -> bool {
        match instr.op() {
            JOp::J => {
                const TOP_BYTE_MASK: u32 = 0xF0000000;
                let top_byte = self.curr_state.pc & TOP_BYTE_MASK;
                self.next_state.pc = top_byte | (instr.target() << 2);
                false
            }
            JOp::JAL => {
                const TOP_BYTE_MASK: u32 = 0xF0000000;
                let top_byte = self.curr_state.pc & TOP_BYTE_MASK;
                self.next_state.pc = top_byte | (instr.target() << 2);
                self.next_state.regs[31] = self.curr_state.pc + 4;
                false
            }
        }
    }

    fn process_itype_instruction(&mut self, instr: &IType) -> bool {
        match instr.op() {
            IOp::BEQ => {
                let ext_off = sign_extend32(instr.imm() << 2, 18);
                let new_addr = self.curr_state.pc as i32 + ext_off;
                if self.curr_state.regs[instr.rs() as usize]
                    == self.curr_state.regs[instr.rt() as usize]
                {
                    self.next_state.pc = new_addr as u32;
                    return false;
                }
                true
            }
            IOp::BNE => {
                let ext_off = sign_extend32(instr.imm() << 2, 18);
                let new_addr = self.curr_state.pc as i32 + ext_off;
                if self.curr_state.regs[instr.rs() as usize]
                    != self.curr_state.regs[instr.rt() as usize]
                {
                    self.next_state.pc = new_addr as u32;
                    return false;
                }
                true
            }
            IOp::BLEZ => {
                let ext_off = sign_extend32(instr.imm() << 2, 18);
                let new_addr = self.curr_state.pc as i32 + ext_off;
                let val = self.curr_state.regs[instr.rs() as usize] as i32;
                if val <= 0 {
                    self.next_state.pc = new_addr as u32;
                    return false;
                }
                true
            }
            IOp::BGEZ => {
                let ext_off = sign_extend32(instr.imm() << 2, 18);
                let new_addr = self.curr_state.pc as i32 + ext_off;
                let val = self.curr_state.regs[instr.rs() as usize] as i32;
                if val >= 0 {
                    self.next_state.pc = new_addr as u32;
                    return false;
                }
                true
            }
            IOp::BGTZ => {
                let ext_off = sign_extend32(instr.imm() << 2, 18);
                let new_addr = self.curr_state.pc as i32 + ext_off;
                let val = self.curr_state.regs[instr.rs() as usize] as i32;
                if val > 0 {
                    self.next_state.pc = new_addr as u32;
                    return false;
                }
                true
            }
            IOp::BLTZ => {
                let ext_off = sign_extend32(instr.imm() << 2, 18);
                let new_addr = self.curr_state.pc as i32 + ext_off;
                let val = self.curr_state.regs[instr.rs() as usize] as i32;
                if val < 0 {
                    self.next_state.pc = new_addr as u32;
                    return false;
                }
                true
            }
            IOp::BLTZAL => {
                let ext_off = sign_extend32(instr.imm() << 2, 18);
                let new_addr = self.curr_state.pc as i32 + ext_off;
                let val = self.curr_state.regs[instr.rs() as usize] as i32;
                self.curr_state.regs[31] = self.curr_state.pc + 4;
                if val < 0 {
                    self.next_state.pc = new_addr as u32;
                    return false;
                }
                true
            }
            IOp::BGEZAL => {
                let ext_off = sign_extend32(instr.imm() << 2, 18);
                let new_addr = self.curr_state.pc as i32 + ext_off;
                let val = self.curr_state.regs[instr.rs() as usize] as i32;
                self.curr_state.regs[31] = self.curr_state.pc + 4;
                if val >= 0 {
                    self.next_state.pc = new_addr as u32;
                    return false;
                }
                true
            }

            IOp::ADDI | IOp::ADDIU => {
                let signed_imm = sign_extend32(instr.imm(), 16);
                self.next_state.regs[instr.rt() as usize] =
                    (self.curr_state.regs[instr.rs() as usize] as i32 + signed_imm) as u32;
                true
            }
            IOp::SLTI => {
                let signed_imm = sign_extend32(instr.imm(), 16);
                if (self.curr_state.regs[instr.rs() as usize] as i32) < signed_imm {
                    self.next_state.regs[instr.rt() as usize] = 1;
                } else {
                    self.next_state.regs[instr.rt() as usize] = 0;
                }
                true
            }
            IOp::SLTIU => {
                let signed_imm = sign_extend32(instr.imm(), 16);
                if self.curr_state.regs[instr.rs() as usize] < signed_imm as u32 {
                    self.next_state.regs[instr.rt() as usize] = 1;
                } else {
                    self.next_state.regs[instr.rt() as usize] = 0;
                }
                true
            }
            IOp::ANDI => {
                self.next_state.regs[instr.rt() as usize] =
                    self.curr_state.regs[instr.rs() as usize] & instr.imm();
                true
            }
            IOp::ORI => {
                self.next_state.regs[instr.rt() as usize] =
                    self.curr_state.regs[instr.rs() as usize] | instr.imm();
                true
            }
            IOp::XORI => {
                self.next_state.regs[instr.rt() as usize] =
                    self.curr_state.regs[instr.rs() as usize] ^ instr.imm();
                true
            }
            IOp::LUI => {
                self.next_state.regs[instr.rt() as usize] = instr.imm() << 16;
                true
            }
            IOp::LB => {
                let offset = sign_extend32(instr.imm(), 16);
                let address = self.curr_state.regs[instr.rs() as usize] as i32 + offset;
                let byte = self
                    .mem_read_8(address as usize)
                    .expect("Cannot read from invalid address");
                self.next_state.regs[instr.rt() as usize] = sign_extend32(byte as u32, 8) as u32;
                true
            }
            IOp::LH => {
                let offset = sign_extend32(instr.imm(), 16);
                let address = self.curr_state.regs[instr.rs() as usize] as i32 + offset;
                let halfword = self
                    .mem_read_16(address as usize)
                    .expect("Cannot read from invalid address");
                self.next_state.regs[instr.rt() as usize] =
                    sign_extend32(halfword as u32, 16) as u32;
                true
            }
            IOp::LW => {
                let offset = sign_extend32(instr.imm(), 16);
                let address = self.curr_state.regs[instr.rs() as usize] as i32 + offset;
                let word = self
                    .mem_read_32(address as usize)
                    .expect("Cannot read from invalid address");
                self.next_state.regs[instr.rt() as usize] = word;
                true
            }
            IOp::LBU => {
                let offset = sign_extend32(instr.imm(), 16);
                let address = self.curr_state.regs[instr.rs() as usize] as i32 + offset;
                let byte = self
                    .mem_read_8(address as usize)
                    .expect("Cannot read from invalid address");
                self.next_state.regs[instr.rt() as usize] = byte as u32;
                true
            }
            IOp::LHU => {
                let offset = sign_extend32(instr.imm(), 16);
                let address = self.curr_state.regs[instr.rs() as usize] as i32 + offset;
                let halfword = self
                    .mem_read_16(address as usize)
                    .expect("Cannot read from invalid address");
                self.next_state.regs[instr.rt() as usize] = halfword as u32;
                true
            }
            IOp::SB => {
                let offset = sign_extend32(instr.imm(), 16);
                let address = self.curr_state.regs[instr.rs() as usize] as i32 + offset;
                const MASK: u32 = 0xFF;
                let written = self.mem_write_32(
                    address as usize,
                    self.curr_state.regs[instr.rt() as usize] & MASK,
                );
                assert!(written);
                true
            }
            IOp::SH => {
                let offset = sign_extend32(instr.imm(), 16);
                let address = self.curr_state.regs[instr.rs() as usize] as i32 + offset;
                const MASK: u32 = 0xFFFF;
                let written = self.mem_write_32(
                    address as usize,
                    self.curr_state.regs[instr.rt() as usize] & MASK,
                );
                assert!(written);
                true
            }
            IOp::SW => {
                let offset = sign_extend32(instr.imm(), 16);
                let address = self.curr_state.regs[instr.rs() as usize] as i32 + offset;
                let written =
                    self.mem_write_32(address as usize, self.curr_state.regs[instr.rt() as usize]);
                assert!(written);
                true
            }
        }
    }

    fn process_rtype_instruction(&mut self, instr: &RType) -> bool {
        match instr.op() {
            ROp::SLL => {
                self.next_state.regs[instr.rd() as usize] =
                    self.curr_state.regs[instr.rt() as usize] << instr.shamt();
                true
            }
            ROp::SRL => {
                self.next_state.regs[instr.rd() as usize] =
                    self.curr_state.regs[instr.rt() as usize] >> instr.shamt();
                true
            }
            ROp::SRA => {
                self.next_state.regs[instr.rd() as usize] =
                    ((self.curr_state.regs[instr.rt() as usize] as i32) >> instr.shamt()) as u32;
                true
            }
            ROp::SLLV => {
                const MASK: u32 = 0x1F;
                let shift = self.curr_state.regs[instr.rs() as usize] & MASK;
                self.next_state.regs[instr.rd() as usize] =
                    self.curr_state.regs[instr.rt() as usize] << shift;
                true
            }
            ROp::SRLV => {
                const MASK: u32 = 0x1F;
                let shift = self.curr_state.regs[instr.rs() as usize] & MASK;
                self.next_state.regs[instr.rd() as usize] =
                    self.curr_state.regs[instr.rt() as usize] >> shift;
                true
            }
            ROp::SRAV => {
                const MASK: u32 = 0x1F;
                let shift = self.curr_state.regs[instr.rs() as usize] & MASK;
                self.next_state.regs[instr.rd() as usize] =
                    ((self.curr_state.regs[instr.rt() as usize] as i32) >> shift) as u32;
                true
            }
            ROp::JR => {
                self.next_state.pc = self.curr_state.regs[instr.rs() as usize];
                false
            }
            ROp::JALR => {
                self.next_state.pc = self.curr_state.regs[instr.rs() as usize];
                self.curr_state.regs[instr.rd() as usize] = self.curr_state.pc + 4;
                false
            }
            ROp::ADD | ROp::ADDU => {
                self.next_state.regs[instr.rd() as usize] = self.curr_state.regs
                    [instr.rs() as usize]
                    + self.curr_state.regs[instr.rt() as usize];
                true
            }
            ROp::SUB | ROp::SUBU => {
                let first = self.curr_state.regs[instr.rs() as usize] as i32;
                let second = self.curr_state.regs[instr.rt() as usize] as i32;
                self.next_state.regs[instr.rd() as usize] = (first - second) as u32;
                true
            }
            ROp::AND => {
                self.next_state.regs[instr.rd() as usize] = self.curr_state.regs
                    [instr.rs() as usize]
                    & self.curr_state.regs[instr.rt() as usize];
                true
            }
            ROp::OR => {
                self.next_state.regs[instr.rd() as usize] = self.curr_state.regs
                    [instr.rs() as usize]
                    | self.curr_state.regs[instr.rt() as usize];
                true
            }
            ROp::XOR => {
                self.next_state.regs[instr.rd() as usize] = self.curr_state.regs
                    [instr.rs() as usize]
                    ^ self.curr_state.regs[instr.rt() as usize];
                true
            }
            ROp::NOR => {
                self.next_state.regs[instr.rd() as usize] = !(self.curr_state.regs
                    [instr.rs() as usize]
                    | self.curr_state.regs[instr.rt() as usize]);
                true
            }
            ROp::SLT => {
                let first = self.curr_state.regs[instr.rs() as usize] as i32;
                let second = self.curr_state.regs[instr.rt() as usize] as i32;
                if first < second {
                    self.next_state.regs[instr.rd() as usize] = 1;
                } else {
                    self.next_state.regs[instr.rd() as usize] = 0;
                }
                true
            }
            ROp::SLTU => {
                let first = self.curr_state.regs[instr.rs() as usize];
                let second = self.curr_state.regs[instr.rt() as usize];
                if first < second {
                    self.next_state.regs[instr.rd() as usize] = 1;
                } else {
                    self.next_state.regs[instr.rd() as usize] = 0;
                }
                true
            }
            ROp::MULT => {
                let first = self.curr_state.regs[instr.rs() as usize] as i64;
                let second = self.curr_state.regs[instr.rt() as usize] as i64;
                let product = (first * second) as u64;
                const LOWER_MASK: u64 = (!(0 as u32)) as u64;
                const UPPER_MASK: u64 = LOWER_MASK << 32;
                self.next_state.hi = ((product & UPPER_MASK) >> 32) as u32;
                self.next_state.lo = (product & LOWER_MASK) as u32;
                true
            }
            ROp::MULTU => {
                let first = self.curr_state.regs[instr.rs() as usize] as u64;
                let second = self.curr_state.regs[instr.rt() as usize] as u64;
                let product = first * second;
                const LOWER_MASK: u64 = (!(0 as u32)) as u64;
                const UPPER_MASK: u64 = LOWER_MASK << 32;
                self.next_state.hi = ((product & UPPER_MASK) >> 32) as u32;
                self.next_state.lo = (product & LOWER_MASK) as u32;
                true
            }
            ROp::DIV => {
                let first = self.curr_state.regs[instr.rs() as usize] as i64;
                let second = self.curr_state.regs[instr.rt() as usize] as i64;
                let product = (first / second) as u64;
                const LOWER_MASK: u64 = (!(0 as u32)) as u64;
                const UPPER_MASK: u64 = LOWER_MASK << 32;
                self.next_state.hi = ((product & UPPER_MASK) >> 32) as u32;
                self.next_state.lo = (product & LOWER_MASK) as u32;
                true
            }
            ROp::DIVU => {
                let first = self.curr_state.regs[instr.rs() as usize] as u64;
                let second = self.curr_state.regs[instr.rt() as usize] as u64;
                let product = first / second;
                const LOWER_MASK: u64 = (!(0 as u32)) as u64;
                const UPPER_MASK: u64 = LOWER_MASK << 32;
                self.next_state.hi = ((product & UPPER_MASK) >> 32) as u32;
                self.next_state.lo = (product & LOWER_MASK) as u32;
                true
            }
            ROp::MFHI => {
                self.next_state.regs[instr.rd() as usize] = self.curr_state.hi;
                true
            }
            ROp::MFLO => {
                self.next_state.regs[instr.rd() as usize] = self.curr_state.lo;
                true
            }
            ROp::MTHI => {
                self.next_state.hi = self.curr_state.regs[instr.rs() as usize];
                true
            }
            ROp::MTLO => {
                self.next_state.lo = self.curr_state.regs[instr.rs() as usize];
                true
            }
            ROp::SYSCALL => {
                if self.curr_state.regs[instr.rd() as usize] == 0xA {
                    self.run_bit = false;
                }
                true
            }
        }
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

    pub fn step(&mut self) {
        self.run(1);
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

        writeln!(out, "\nMemory content [{:#010X}..{:#010X}] :", start, stop)?;
        writeln!(out, "-----------------------------------------")?;
        address = start;
        while address <= stop {
            if let Some(value) = self.mem_read_32(address) {
                writeln!(
                    out,
                    "    {:#010X}  ({}) : {:#010X}",
                    address, address, value
                )?;
            } else {
                writeln!(
                    out,
                    "    {:#010X}  ({}) : <undefined address>",
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
        writeln!(out, "PC                : {:#010X}", self.curr_state.pc)?;
        writeln!(out, "Registers:")?;
        for (i, reg) in self.curr_state.regs.iter().enumerate() {
            writeln!(out, "R{}: {:#010X}", i, reg)?;
        }
        writeln!(out, "HI: {:#010X}", self.curr_state.hi)?;
        writeln!(out, "LO: {:#010X}", self.curr_state.lo)?;
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

fn sign_extend32(data: u32, size: u32) -> i32 {
    assert!(size <= 32);
    ((data << (32 - size)) as i32) >> (32 - size)
}
