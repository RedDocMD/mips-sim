pub const MIPS_REGS: usize = 32;

struct CpuState {
    PC: u32,
    REGS: [u32; MIPS_REGS],
    HI: u32,
    LO: u32,
}

struct MemRegion {
    start: usize,
    size: usize,
    mem: Vec<u8>,
}

struct MipsComputer {
    curr_state: CpuState,
    next_state: CpuState,
    run_bit: bool,
    instr_cnt: i32,
    memory: [MemRegion; 5],
}

impl CpuState {
    fn new() -> Self {
        Self {
            PC: 0,
            REGS: [0; MIPS_REGS],
            HI: 0,
            LO: 0,
        }
    }
}

impl MemRegion {
    fn new(start: usize, size: usize) -> Self {
        Self {
            start,
            size,
            mem: Vec::with_capacity(size),
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
    fn new() -> Self {
        Self {
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
        }
    }
}
