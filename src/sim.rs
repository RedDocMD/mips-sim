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
        let instr = self.mem_read_32(self.curr_state.pc as usize);
        if let Some(instr) = instr {
            if instr == 0 {
                self.run_bit = false;
            } else {
                let instr = parse_instr(instr);
                println!("{:?}", instr);
                self.next_state.pc = self.curr_state.pc + 4;
            }
        } else {
            self.run_bit = false;
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

#[derive(Debug)]
struct JType {
    opcode: u32,
    target: u32,
    op: JOp,
}

#[derive(Debug)]
struct IType {
    opcode: u32,
    rs: u32,
    rt: u32,
    imm: u32,
    op: IOp,
}

#[derive(Debug)]
struct RType {
    opcode: u32,
    rs: u32,
    rt: u32,
    rd: u32,
    shamt: u32,
    funct: u32,
    op: ROp,
}

#[derive(Debug)]
enum Instr {
    JType(JType),
    IType(IType),
    RType(RType),
}

#[derive(Debug)]
enum JOp {
    J,
    JAL,
}

#[derive(Debug)]
enum IOp {
    BEQ,
    BNE,
    BLEZ,
    BGTZ,
    ADDI,
    ADDIU,
    SLTI,
    SLTIU,
    ANDI,
    ORI,
    XORI,
    LUI,
    LB,
    LH,
    LW,
    LBU,
    LHU,
    SB,
    SH,
    SW,
    BLTZ,
    BGEZ,
    BLTZAL,
    BGEZAL,
}

#[derive(Debug)]
enum ROp {
    SLL,
    SRL,
    SRA,
    SLLV,
    SRLV,
    SRAV,
    JR,
    JALR,
    ADD,
    ADDU,
    SUB,
    SUBU,
    AND,
    OR,
    XOR,
    NOR,
    SLT,
    SLTU,
    MULT,
    MULTU,
    DIV,
    DIVU,
    MFHI,
    MFLO,
    MTHI,
    MTLO,
    SYSCALL,
}

// Extract the top 6 bits
fn extract_opcode(instr: u32) -> u32 {
    const MASK: u32 = 0xFC000000;
    const POS: u32 = 26;

    (instr & MASK) >> POS
}

fn parse_instr(instr: u32) -> Instr {
    let opcode = extract_opcode(instr);
    match opcode {
        0x2 => Instr::JType(parse_jump_instr(instr, JOp::J)),
        0x3 => Instr::JType(parse_jump_instr(instr, JOp::JAL)),
        0x4 => Instr::IType(parse_immediate_instr(instr, IOp::BEQ)),
        0x5 => Instr::IType(parse_immediate_instr(instr, IOp::BNE)),
        0x6 => Instr::IType(parse_immediate_instr(instr, IOp::BLEZ)),
        0x7 => Instr::IType(parse_immediate_instr(instr, IOp::BGTZ)),
        0x8 => Instr::IType(parse_immediate_instr(instr, IOp::ADDI)),
        0x9 => Instr::IType(parse_immediate_instr(instr, IOp::ADDIU)),
        0xA => Instr::IType(parse_immediate_instr(instr, IOp::SLTI)),
        0xB => Instr::IType(parse_immediate_instr(instr, IOp::SLTIU)),
        0xC => Instr::IType(parse_immediate_instr(instr, IOp::ANDI)),
        0xD => Instr::IType(parse_immediate_instr(instr, IOp::ORI)),
        0xE => Instr::IType(parse_immediate_instr(instr, IOp::XORI)),
        0xF => Instr::IType(parse_immediate_instr(instr, IOp::LUI)),
        0x20 => Instr::IType(parse_immediate_instr(instr, IOp::LB)),
        0x21 => Instr::IType(parse_immediate_instr(instr, IOp::LH)),
        0x23 => Instr::IType(parse_immediate_instr(instr, IOp::LW)),
        0x24 => Instr::IType(parse_immediate_instr(instr, IOp::LBU)),
        0x25 => Instr::IType(parse_immediate_instr(instr, IOp::LHU)),
        0x28 => Instr::IType(parse_immediate_instr(instr, IOp::SB)),
        0x29 => Instr::IType(parse_immediate_instr(instr, IOp::SH)),
        0x2B => Instr::IType(parse_immediate_instr(instr, IOp::SW)),
        0x1 => Instr::IType(parse_immediate_instr_and_op(instr)),
        0x0 => Instr::RType(parse_register_instr(instr)),
        _ => panic!("Unknown instruction!"),
    }
}

fn parse_jump_instr(instr: u32, op: JOp) -> JType {
    const MASK: u32 = 0x3FFFFFF;
    JType {
        opcode: extract_opcode(instr),
        target: instr & MASK,
        op,
    }
}

fn parse_immediate_instr(instr: u32, op: IOp) -> IType {
    const RS_MASK: u32 = 0x3E00000;
    const RS_SHIFT: u32 = 21;
    const RT_MASK: u32 = 0x1F0000;
    const RT_SHIFT: u32 = 16;
    const IMM_MASK: u32 = 0xFFFF;
    let rs = (instr & RS_MASK) >> RS_SHIFT;
    let rt = (instr & RT_MASK) >> RT_SHIFT;
    let imm = instr & IMM_MASK;
    IType {
        rs,
        rt,
        imm,
        opcode: extract_opcode(instr),
        op,
    }
}

fn parse_immediate_instr_and_op(instr: u32) -> IType {
    const RS_MASK: u32 = 0x3E00000;
    const RS_SHIFT: u32 = 21;
    const RT_MASK: u32 = 0x1F0000;
    const RT_SHIFT: u32 = 16;
    const IMM_MASK: u32 = 0xFFFF;
    let rs = (instr & RS_MASK) >> RS_SHIFT;
    let rt = (instr & RT_MASK) >> RT_SHIFT;
    let imm = instr & IMM_MASK;
    let op = match rt {
        0x0 => IOp::BLTZ,
        0x1 => IOp::BGEZ,
        0x20 => IOp::BLTZAL,
        0x21 => IOp::BGEZAL,
        _ => panic!("Uknown branch instruction for REGIMM"),
    };
    IType {
        rs,
        rt,
        imm,
        opcode: extract_opcode(instr),
        op,
    }
}

fn parse_register_instr(instr: u32) -> RType {
    const RS_MASK: u32 = 0x3E00000;
    const RS_SHIFT: u32 = 21;
    const RT_MASK: u32 = 0x1F0000;
    const RT_SHIFT: u32 = 16;
    const RD_MASK: u32 = 0xF800;
    const RD_SHIFT: u32 = 11;
    const SHAMT_MASK: u32 = 0x7C0;
    const SHAMT_SHIFT: u32 = 6;
    const FUNCT_MASK: u32 = 0x3F;
    let rs = (instr & RS_MASK) >> RS_SHIFT;
    let rt = (instr & RT_MASK) >> RT_SHIFT;
    let rd = (instr & RD_MASK) >> RD_SHIFT;
    let shamt = (instr & SHAMT_MASK) >> SHAMT_SHIFT;
    let funct = instr & FUNCT_MASK;

    assert_eq!(extract_opcode(instr), 0);
    let op = match funct {
        0x0 => ROp::SLL,
        0x2 => ROp::SRL,
        0x3 => ROp::SRA,
        0x4 => ROp::SLLV,
        0x6 => ROp::SRLV,
        0x7 => ROp::SRAV,
        0x8 => ROp::JR,
        0x9 => ROp::JALR,
        0x20 => ROp::ADD,
        0x21 => ROp::ADDU,
        0x22 => ROp::SUB,
        0x23 => ROp::SUBU,
        0x24 => ROp::AND,
        0x25 => ROp::OR,
        0x26 => ROp::XOR,
        0x27 => ROp::NOR,
        0x2A => ROp::SLT,
        0x2B => ROp::SLTU,
        0x18 => ROp::MULT,
        0x19 => ROp::MULTU,
        0x1A => ROp::DIV,
        0x1B => ROp::DIVU,
        0x10 => ROp::MFHI,
        0x12 => ROp::MFLO,
        0x11 => ROp::MTHI,
        0x13 => ROp::MTLO,
        0xC => ROp::SYSCALL,
        _ => panic!("Unknown R Type instruction"),
    };

    RType {
        opcode: 0,
        rs,
        rt,
        rd,
        shamt,
        funct,
        op,
    }
}
