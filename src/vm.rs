use std::ops::{Index, IndexMut};

const MEMORY_SIZE: usize = 1 << 16;

#[derive(Debug, Clone, PartialEq, Eq)]
struct Memory {
    ram: [u16; MEMORY_SIZE],
}

impl Memory {
    fn new() -> Self {
        Self {
            ram: [0; MEMORY_SIZE],
        }
    }

    fn load(&mut self, block: &[u16], offset: u16) {
        let start = offset as usize;
        let end = start + block.len();
        self.ram[start..end].copy_from_slice(block);
    }
}

impl Index<u16> for Memory {
    type Output = u16;

    fn index(&self, index: u16) -> &Self::Output {
        &self.ram[index as usize]
    }
}

impl IndexMut<u16> for Memory {
    fn index_mut(&mut self, index: u16) -> &mut Self::Output {
        &mut self.ram[index as usize]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct RegisterIndex(u8);

impl Default for RegisterIndex {
    fn default() -> Self {
        RegisterIndex(0)
    }
}

const REGISTER_COUNT: usize = 10;
const REG_PC: RegisterIndex = RegisterIndex(8);
const REG_COND: RegisterIndex = RegisterIndex(9);

#[derive(Debug, Clone, PartialEq, Eq)]
struct RegisterCluster {
    registers: [u16; REGISTER_COUNT],
}

impl Default for RegisterCluster {
    fn default() -> Self {
        RegisterCluster {
            registers: [0; REGISTER_COUNT],
        }
    }
}

impl Index<RegisterIndex> for RegisterCluster {
    type Output = u16;

    fn index(&self, index: RegisterIndex) -> &Self::Output {
        &self.registers[index.0 as usize]
    }
}

impl IndexMut<RegisterIndex> for RegisterCluster {
    fn index_mut(&mut self, index: RegisterIndex) -> &mut Self::Output {
        &mut self.registers[index.0 as usize]
    }
}

trait BitTool {
    fn use_immediate_mode(self) -> bool;
    fn read_dr(self) -> RegisterIndex;
    fn read_sr1(self) -> RegisterIndex;
    fn read_sr2(self) -> RegisterIndex;
    fn read_imme5(self) -> u16;
    fn read_pc_offset9(self) -> u16;
}

impl BitTool for u16 {
    fn use_immediate_mode(self) -> bool {
        0b0000_0000_0001_0000 & self == 0b0000_0000_0001_0000
    }

    fn read_dr(self) -> RegisterIndex {
        let register_value = (self >> 9) & 0x07;
        RegisterIndex(register_value as u8)
    }

    fn read_sr1(self) -> RegisterIndex {
        let register_value = (self >> 5) & 0x07;
        RegisterIndex(register_value as u8)
    }

    fn read_sr2(self) -> RegisterIndex {
        let register_value = self & 0x07;
        RegisterIndex(register_value as u8)
    }

    fn read_imme5(self) -> u16 {
        self & 0x1f
    }

    fn read_pc_offset9(self) -> u16 {
        self & 0x7f
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum Opcode {
    Branch,
    Add {
        dr: RegisterIndex,
        sr1: RegisterIndex,
        sr2: RegisterIndex,
    },
    AddImmediate {
        dr: RegisterIndex,
        sr1: RegisterIndex,
        imm5: u16,
    },
    Load {
        dr: RegisterIndex,
        pc_offfset9: u16,
    },
    Store {
        sr: RegisterIndex,
        pc_offset9: u16,
    },
    JumpRegister,
    And,
    LoadRegister,
    StoreRegister,
    Unused,
    Not,
    LoadIndirect,
    StoreIndirect,
    Jump,
    Reserved,
    LoadEffectiveAddress,
    Trap,
}

impl From<u16> for Opcode {
    fn from(value: u16) -> Self {
        let op = value >> 12;
        match op {
            0b0001 => {
                if value.use_immediate_mode() {
                    Opcode::AddImmediate {
                        dr: value.read_dr(),
                        sr1: value.read_sr1(),
                        imm5: value.read_imme5(),
                    }
                } else {
                    Opcode::Add {
                        dr: value.read_dr(),
                        sr1: value.read_sr1(),
                        sr2: value.read_sr2(),
                    }
                }
            }
            0b0010 => Opcode::Load {
                dr: value.read_dr(),
                pc_offfset9: value.read_pc_offset9(),
            },
            0b0011 => Opcode::Store {
                sr: value.read_dr(),
                pc_offset9: value.read_pc_offset9(),
            },
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Machine {
    mem: Memory,
    registers: RegisterCluster,
}

impl Machine {
    pub fn new() -> Self {
        Self {
            mem: Memory::new(),
            registers: RegisterCluster::default(),
        }
    }

    pub fn load(&mut self, block: &[u16], offset: u16) {
        self.mem.load(block, offset);
    }

    pub fn step(&mut self) {
        let instr = self.mem[self.registers[REG_PC]];
        self.registers[REG_PC] += 2;
        let opcode = Opcode::from(instr);

        match opcode {
            Opcode::Add { dr, sr1, sr2 } => {
                self.registers[dr] = self.registers[sr1] + self.registers[sr2];
            }
            Opcode::AddImmediate { dr, sr1, imm5 } => {
                self.registers[dr] = self.registers[sr1] + imm5;
            }
            Opcode::Load { dr, pc_offfset9 } => {
                self.registers[dr] = self.mem[self.registers[REG_PC] + pc_offfset9];
            }
            Opcode::Store { sr, pc_offset9 } => {
                self.mem[self.registers[REG_PC] + pc_offset9] = self.registers[sr];
            }
            _ => unreachable!(),
        }
    }
}
