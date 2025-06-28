use crate::ast::{self, CondOp, OwnedInstruction};
use crate::ast::{Instruction, LabOrImm, Register};
mod atomic_change_display;

/// Vec(instruction_pointer, instruction)
pub type Disassembly = Vec<(i64, OwnedInstruction)>;

/// Vec<(instruction_number, changes)>
/// (id, change) in Log means that Diassembly\[id\] caused change
pub type Log = Vec<(usize, AtomicChange)>;

pub enum AtomicChange {
    /// A change in the value of a register.
    Register { reg: ast::Register, value: i64 },
    /// A change in the value of a memory location.
    Memory { addr: i64, value: i64 },
    /// A change in the instruction pointer.
    InstructionPointer { ip: i64 },
    /// A change in the condition code.
    ConditionCode { cc: u8 },
    /// A change in the simulator state.
    State { status: Status },
}

#[derive(Debug, Clone, PartialEq)]
pub enum Status {
    Running,
    Halted,
    Error(String),
}

pub struct Simulator<'a, const MEM_SIZE: usize> {
    /// The current values of the registers.
    pub registers: [i64; 13], // Rax, Rbx, Rcx, Rdx, Rdi, Rsi, Rsp, Rbp, R8, R9, R10, R11, R12
    /// The current value of the instruction pointer.
    pub instruction_pointer: i64,
    /// The memory state.
    pub memory: [i64; MEM_SIZE],
    /// The source code being simulated.
    pub source: &'a [u8],
    /// The current state of the simulator.
    pub state: Status,
    /// The 4 bit condition code
    pub condition_code: u8, // 4 bits for condition codes

    /// Disassembly of instructions executed
    pub disassembly: Disassembly,
    /// Log of changes made during execution
    pub log: Log,

    /// Index of the next change to next_to_commit
    next_to_commit: usize,
}

impl<'a, const MEM_SIZE: usize> Simulator<'a, MEM_SIZE> {
    /// Creates a new simulator state with all registers set to 0 and memory initialized to 0.
    pub fn new(src: &'a [u8]) -> Self {
        Self {
            registers: [0; 13],
            instruction_pointer: 0,
            memory: [0; MEM_SIZE],
            source: src,
            state: Status::Running,
            condition_code: 0,
            disassembly: Disassembly::new(),
            log: Log::new(),
            next_to_commit: 0,
        }
    }

    /// Resets the simulator state to its initial values.
    pub fn reset(&mut self) {
        self.registers = [0; 13];
        self.instruction_pointer = 0;
        self.memory = [0; MEM_SIZE];
    }

    /// Applies all the uncommitted changes in the log to the simulator state.
    fn apply_changes(&mut self) {
        for (_, change) in self.log[self.next_to_commit..].iter() {
            match change {
                &AtomicChange::Register { reg, value } => {
                    self.registers[reg as usize] = value;
                }
                &AtomicChange::Memory { addr, value } => {
                    if addr >= 0 && (addr as usize) < MEM_SIZE {
                        self.memory[addr as usize] = value;
                    } else {
                        self.state =
                            Status::Error(format!("Memory address out of bounds: {}", addr));
                    }
                }
                &AtomicChange::InstructionPointer { ip } => {
                    self.instruction_pointer = ip;
                }
                &AtomicChange::ConditionCode { cc } => {
                    self.condition_code = cc;
                }
                AtomicChange::State { status } => {
                    self.state = status.clone();
                }
            }
        }
        self.next_to_commit = self.log.len();
    }

    fn condition_ok(&self, cond: CondOp) -> bool {
        match cond {
            CondOp::Uncon => true,
            _ => todo!(),
        }
    }
    /// Executes the given instruction until it halts
    pub fn run_single(&mut self) {
        let fetch_result = self.fetch_decode();
        if let Err(e) = fetch_result {
            self.state = Status::Error(e);
            return;
        }
        let instruction = fetch_result.unwrap();
        self.disassembly
            .push((self.instruction_pointer, instruction));

        let instr = &self.disassembly.last().unwrap().1;
        let id = self.disassembly.len() - 1;
        match &instr {
            Instruction::Halt => {
                self.log.push((
                    id,
                    (AtomicChange::State {
                        status: Status::Halted,
                    }),
                ));
            }
            Instruction::Nop => {
                self.log.push((
                    id,
                    AtomicChange::InstructionPointer {
                        ip: self.instruction_pointer + 1,
                    },
                ));
            }
            Instruction::Cmov(cond, r1, r2) => {
                if self.condition_ok(*cond) {
                    self.log.push((
                        id,
                        AtomicChange::Register {
                            reg: *r2,
                            value: self.registers[*r1 as usize],
                        },
                    ));
                };

                self.log.push((
                    id,
                    AtomicChange::InstructionPointer {
                        ip: self.instruction_pointer + 2,
                    },
                ));
            }
            Instruction::Irmov(imm, regs) => {
                let LabOrImm::Immediate(imm_val) = imm else {
                    self.state = Status::Error("Invalid immediate value".to_string());
                    return;
                };
                self.log.push((
                    id,
                    AtomicChange::Register {
                        reg: *regs,
                        value: *imm_val,
                    },
                ));
                self.log.push((
                    id,
                    AtomicChange::InstructionPointer {
                        ip: self.instruction_pointer + 10,
                    },
                ));
            }
            Instruction::Rmmov(src, disp, dst) => {
                let value = self.registers[*src as usize];
                let addr = disp + self.registers[*dst as usize];

                if addr < 0 || (addr as usize) >= MEM_SIZE {
                    self.state = Status::Error(format!("Memory address out of bounds: {}", addr));
                    return;
                }

                self.log.push((id, AtomicChange::Memory { addr, value }));
                self.log.push((
                    id,
                    AtomicChange::InstructionPointer {
                        ip: self.instruction_pointer + 10,
                    },
                ));
            }
            // Handle other instructions...
            _ => todo!(),
        }
        self.apply_changes();
    }

    fn fetch_decode_regs(&self, ptr: i64) -> Result<(Register, Register), String> {
        let byte = self
            .source
            .get(ptr as usize)
            .ok_or_else(|| format!("IP Out of Range: {}", ptr))?;

        let Ok(reg_a) = Register::try_from(*byte >> 4) else {
            return Err(format!("Invalid register A: {}", byte >> 4));
        };
        let Ok(reg_b) = Register::try_from(*byte & 0x0F) else {
            return Err(format!("Invalid register B: {}", byte & 0x0F));
        };
        Ok((reg_a, reg_b))
    }

    fn fetch_decode_regb(&self, ptr: i64) -> Result<Register, String> {
        let byte = self
            .source
            .get(ptr as usize)
            .ok_or_else(|| format!("IP Out of Range: {}", ptr))?;

        let Ok(reg_b) = Register::try_from(*byte & 0x0F) else {
            return Err(format!("Invalid register B: {}", byte & 0x0F));
        };

        Ok(reg_b)
    }

    fn fetch_decode_imm(&self, ptr: i64) -> Result<i64, String> {
        let bytes = self
            .source
            .get(ptr as usize..ptr as usize + 8)
            .ok_or_else(|| format!("IP Out of Range: {}", ptr))?;

        if bytes.len() < 8 {
            return Err(format!("Immediate value too short at IP: {}", ptr));
        }

        let mut imm = 0i64;
        for (i, &byte) in bytes.iter().enumerate() {
            // Little-endian order
            imm |= (byte as i64) << (i * 8);
        }
        Ok(imm)
    }

    fn fetch_decode(&self) -> Result<OwnedInstruction, String> {
        let byte0 = self
            .source
            .get(self.instruction_pointer as usize)
            .ok_or_else(|| format!("IP Out of Range: {}", self.instruction_pointer))?;

        let opcode = byte0 >> 4;
        let _func = byte0 & 0x0F;

        match opcode {
            0x0 => Ok(Instruction::Halt),
            0x1 => Ok(Instruction::Nop),
            0x2 => {
                let (r1, r2) = self.fetch_decode_regs(self.instruction_pointer + 1)?;
                Ok(Instruction::Cmov(CondOp::Uncon, r1, r2))
            }
            0x3 => {
                let r2 = self.fetch_decode_regb(self.instruction_pointer + 1)?;
                let imm = self.fetch_decode_imm(self.instruction_pointer + 2)?;
                Ok(Instruction::Irmov(LabOrImm::Immediate(imm), r2))
            }
            0x4 => {
                let (r1, r2) = self.fetch_decode_regs(self.instruction_pointer + 1)?;
                let disp = self.fetch_decode_imm(self.instruction_pointer + 2)?;
                Ok(Instruction::Rmmov(r1, disp, r2))
            }
            // Add more opcodes as needed
            _ => Err(format!("Unknown opcode: {:#x}", opcode)),
        }
    }

    pub fn is_halted(&self) -> bool {
        matches!(self.state, Status::Halted)
    }
}
