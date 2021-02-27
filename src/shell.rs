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
        if self.contains_address(address) {
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
}
