#[derive(Debug)]
pub struct JType {
    opcode: u32,
    target: u32,
    op: JOp,
}

#[derive(Debug)]
pub struct IType {
    opcode: u32,
    rs: u32,
    rt: u32,
    imm: u32,
    op: IOp,
}

#[derive(Debug)]
pub struct RType {
    opcode: u32,
    rs: u32,
    rt: u32,
    rd: u32,
    shamt: u32,
    funct: u32,
    op: ROp,
}

#[derive(Debug)]
pub enum Instr {
    JType(JType),
    IType(IType),
    RType(RType),
}

#[derive(Debug)]
pub enum JOp {
    J,
    JAL,
}

#[derive(Debug)]
pub enum IOp {
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
pub enum ROp {
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

pub fn parse_instr(instr: u32) -> Instr {
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
