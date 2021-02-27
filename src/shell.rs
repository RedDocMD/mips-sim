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
