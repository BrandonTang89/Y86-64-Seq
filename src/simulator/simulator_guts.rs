use crate::ast::{self, CondOp, OwnedInstruction};
use crate::ast::{Instruction, LabOrImm, Register};
mod atomic_change_display;
#[cfg(test)]
mod simulator_guts_tests;

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

static CARRY_MASK: u8 = 0b0001; // 4 bits for condition codes
static ZERO_MASK: u8 = 0b0010;
static SIGN_MASK: u8 = 0b0100;
static OVERFLOW_MASK: u8 = 0b1000;

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
        // initialise stack pointer to the end of memory
        let mut registers = [0; 13];
        registers[Register::Rsp as usize] = (MEM_SIZE - 8) as i64; // Stack pointer starts at the end of memory
        Self {
            registers,
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
        self.condition_code = 0;
        self.state = Status::Running;
        self.disassembly.clear();
        self.log.clear();
        self.next_to_commit = 0;
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
        let zero = (self.condition_code & ZERO_MASK) != 0; // Z flag
        let _carry = (self.condition_code & CARRY_MASK) != 0; // C flag  
        let sign = (self.condition_code & SIGN_MASK) != 0; // N flag (negative)
        let overflow = (self.condition_code & OVERFLOW_MASK) != 0; // V flag

        match cond {
            CondOp::Uncon => true,                     // Unconditional
            CondOp::Eq => zero,                        // Equal: Z==1
            CondOp::Ne => !zero,                       // Not equal: Z==0
            CondOp::Ge => sign == overflow,            // Greater or equal: N==V
            CondOp::Lt => sign != overflow,            // Less than: N!=V
            CondOp::Gt => !zero && (sign == overflow), // Greater than: (Z==0) && (N==V)
            CondOp::Le => zero || (sign != overflow),  // Less or equal: (Z==1) || (N!=V)
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
            Instruction::Mrmov(disp, src, dst) => {
                let addr = disp + self.registers[*src as usize];
                if addr < 0 || (addr as usize) >= MEM_SIZE {
                    self.state = Status::Error(format!("Memory address out of bounds: {}", addr));
                    return;
                }
                let value = self.memory[addr as usize];
                self.log
                    .push((id, AtomicChange::Register { reg: *dst, value }));
                self.log.push((
                    id,
                    AtomicChange::InstructionPointer {
                        ip: self.instruction_pointer + 10,
                    },
                ));
            }
            Instruction::Binop(op, src, dest) => {
                let r1 = self.registers[*src as usize];
                let r2 = self.registers[*dest as usize];

                let original_carry = self.condition_code & CARRY_MASK != 0;
                let original_overflow = self.condition_code & OVERFLOW_MASK != 0;

                let (result, carry, overflow) = match op {
                    ast::BinaryOp::Add => {
                        let (res, car) = r1.overflowing_add(r2);

                        let overflow = if r1 < 0 && r2 < 0 && res >= 0 {
                            true
                        } else if r1 >= 0 && r2 >= 0 && res < 0 {
                            true
                        } else {
                            false
                        };
                        (res, car, overflow)
                    }
                    ast::BinaryOp::Sub => {
                        let nr1 = -r1;
                        let (res, car) = nr1.overflowing_add(r2);
                        let overflow = if nr1 < 0 && r2 < 0 && res >= 0 {
                            true
                        } else if nr1 >= 0 && r2 >= 0 && res < 0 {
                            true
                        } else {
                            false
                        };
                        (res, car, overflow)
                    }
                    ast::BinaryOp::And => {
                        let res = r1 & r2;
                        (res, original_carry, original_overflow)
                    }
                    ast::BinaryOp::Xor => {
                        let res = r1 ^ r2;
                        (res, original_carry, original_overflow)
                    }
                };

                self.log.push((
                    id,
                    AtomicChange::Register {
                        reg: *dest,
                        value: result,
                    },
                ));

                self.log.push((
                    id,
                    AtomicChange::ConditionCode {
                        cc: ((if carry { CARRY_MASK } else { 0 })
                            | (if result == 0 { ZERO_MASK } else { 0 })
                            | (if result < 0 { SIGN_MASK } else { 0 })
                            | (if overflow { OVERFLOW_MASK } else { 0 })),
                    },
                ));

                self.log.push((
                    id,
                    AtomicChange::InstructionPointer {
                        ip: self.instruction_pointer + 2,
                    },
                ));
            }

            Instruction::Call(target) => {
                let LabOrImm::Immediate(imm) = target else {
                    self.state = Status::Error("Invalid immediate value in Call".to_string());
                    return;
                };

                if *imm < 0 || (*imm) >= MEM_SIZE as i64 {
                    self.state = Status::Error(format!("Call address out of bounds: {}", imm));
                    return;
                }

                let new_sp = self.registers[Register::Rsp as usize] - 8;
                if new_sp < 0 || (new_sp as usize) >= MEM_SIZE {
                    self.state = Status::Error(format!("Stack pointer out of bounds: {}", new_sp));
                    return;
                }

                let ret_addr = self.instruction_pointer + 9;

                self.log.push((
                    id,
                    AtomicChange::Memory {
                        addr: new_sp,
                        value: ret_addr,
                    },
                ));

                self.log.push((
                    id,
                    AtomicChange::Register {
                        reg: Register::Rsp,
                        value: new_sp,
                    },
                ));

                self.log
                    .push((id, AtomicChange::InstructionPointer { ip: *imm }));
            }
            Instruction::Ret => {
                let ret_addr = self
                    .memory
                    .get(self.registers[Register::Rsp as usize] as usize)
                    .cloned()
                    .unwrap_or_else(|| {
                        self.state = Status::Error("Return address not found on stack".to_string());
                        -1
                    });
                if ret_addr < 0 || (ret_addr as usize) >= MEM_SIZE {
                    self.state =
                        Status::Error(format!("Return address out of bounds: {}", ret_addr));
                    return;
                }
                let new_sp = self.registers[Register::Rsp as usize] + 8;
                if new_sp < 0 || (new_sp as usize) >= MEM_SIZE {
                    self.state = Status::Error(format!("Stack pointer out of bounds: {}", new_sp));
                    return;
                }

                self.log.push((
                    id,
                    AtomicChange::Register {
                        reg: Register::Rsp,
                        value: new_sp,
                    },
                ));

                self.log
                    .push((id, AtomicChange::InstructionPointer { ip: ret_addr }));
            }
            Instruction::Jmp(cond, target) => {
                let LabOrImm::Immediate(addr) = target else {
                    self.state = Status::Error("Invalid jump target".to_string());
                    return;
                };

                if *addr < 0 || (*addr as usize) >= self.source.len() {
                    self.state = Status::Error(format!("Jump address out of bounds: {}", addr));
                    return;
                }

                let new_ip = if self.condition_ok(*cond) {
                    // Conditional jump taken
                    *addr
                } else {
                    // Conditional jump not taken, continue to next instruction
                    self.instruction_pointer + 9 // 1 byte opcode + 8 bytes immediate
                };

                self.log
                    .push((id, AtomicChange::InstructionPointer { ip: new_ip }));
            }
            Instruction::Push(reg) => {
                let new_sp = self.registers[Register::Rsp as usize] - 8;
                if new_sp < 0 || (new_sp as usize) >= MEM_SIZE {
                    self.state = Status::Error(format!("Stack pointer out of bounds: {}", new_sp));
                    return;
                }

                let value = self.registers[*reg as usize];

                self.log.push((
                    id,
                    AtomicChange::Memory {
                        addr: new_sp,
                        value,
                    },
                ));

                self.log.push((
                    id,
                    AtomicChange::Register {
                        reg: Register::Rsp,
                        value: new_sp,
                    },
                ));

                self.log.push((
                    id,
                    AtomicChange::InstructionPointer {
                        ip: self.instruction_pointer + 2,
                    },
                ));
            }
            Instruction::Pop(reg) => {
                let sp = self.registers[Register::Rsp as usize];
                if sp < 0 || (sp as usize) >= MEM_SIZE {
                    self.state = Status::Error(format!("Stack pointer out of bounds: {}", sp));
                    return;
                }

                let value = self.memory[sp as usize];

                self.log
                    .push((id, AtomicChange::Register { reg: *reg, value }));

                // Only update %rsp if we're not popping to %rsp
                // If we're popping to %rsp, the value from the stack becomes the new %rsp
                if *reg != Register::Rsp {
                    let new_sp = sp + 8;
                    self.log.push((
                        id,
                        AtomicChange::Register {
                            reg: Register::Rsp,
                            value: new_sp,
                        },
                    ));
                }

                self.log.push((
                    id,
                    AtomicChange::InstructionPointer {
                        ip: self.instruction_pointer + 2,
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

    fn fetch_decode_rega(&self, ptr: i64) -> Result<Register, String> {
        let byte = self
            .source
            .get(ptr as usize)
            .ok_or_else(|| format!("IP Out of Range: {}", ptr))?;

        let Ok(reg_a) = Register::try_from(*byte >> 4) else {
            return Err(format!("Invalid register A: {}", byte >> 4));
        };

        Ok(reg_a)
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
        let func = byte0 & 0x0F;

        match opcode {
            0x0 => Ok(Instruction::Halt),
            0x1 => Ok(Instruction::Nop),
            0x2 => {
                let (r_a, r_b) = self.fetch_decode_regs(self.instruction_pointer + 1)?;
                let cond = match func {
                    0x0 => CondOp::Uncon,
                    0x1 => CondOp::Le,
                    0x2 => CondOp::Lt,
                    0x3 => CondOp::Eq,
                    0x4 => CondOp::Ne,
                    0x5 => CondOp::Ge,
                    0x6 => CondOp::Gt,
                    _ => return Err(format!("Invalid condition code function: {}", func)),
                };
                Ok(Instruction::Cmov(cond, r_a, r_b))
            }
            0x3 => {
                let r_b = self.fetch_decode_regb(self.instruction_pointer + 1)?;
                let imm = self.fetch_decode_imm(self.instruction_pointer + 2)?;
                Ok(Instruction::Irmov(LabOrImm::Immediate(imm), r_b))
            }
            0x4 => {
                let (r_a, r_b) = self.fetch_decode_regs(self.instruction_pointer + 1)?;
                let imm = self.fetch_decode_imm(self.instruction_pointer + 2)?;
                Ok(Instruction::Rmmov(r_a, imm, r_b))
            }
            0x5 => {
                let (r_a, r_b) = self.fetch_decode_regs(self.instruction_pointer + 1)?;
                let imm = self.fetch_decode_imm(self.instruction_pointer + 2)?;
                Ok(Instruction::Mrmov(imm, r_b, r_a))
            }
            0x6 => {
                let (r_a, r_b) = self.fetch_decode_regs(self.instruction_pointer + 1)?;
                let op = match func {
                    0x0 => ast::BinaryOp::Add,
                    0x1 => ast::BinaryOp::Sub,
                    0x2 => ast::BinaryOp::And,
                    0x3 => ast::BinaryOp::Xor,
                    _ => return Err(format!("Invalid binary operation function: {}", func)),
                };
                Ok(Instruction::Binop(op, r_a, r_b))
            }
            0x7 => {
                let cond = match func {
                    0x0 => CondOp::Uncon,
                    0x1 => CondOp::Le,
                    0x2 => CondOp::Lt,
                    0x3 => CondOp::Eq,
                    0x4 => CondOp::Ne,
                    0x5 => CondOp::Ge,
                    0x6 => CondOp::Gt,
                    _ => return Err(format!("Invalid jump condition function: {}", func)),
                };
                let imm = self.fetch_decode_imm(self.instruction_pointer + 1)?;
                Ok(Instruction::Jmp(cond, LabOrImm::Immediate(imm)))
            }
            0x8 => {
                let imm = self.fetch_decode_imm(self.instruction_pointer + 1)?;
                Ok(Instruction::Call(LabOrImm::Immediate(imm)))
            }
            0x9 => Ok(Instruction::Ret),
            0xa => {
                let reg = self.fetch_decode_rega(self.instruction_pointer + 1)?;
                Ok(Instruction::Push(reg))
            }
            0xb => {
                let reg = self.fetch_decode_rega(self.instruction_pointer + 1)?;
                Ok(Instruction::Pop(reg))
            }
            // Add more opcodes as needed
            _ => Err(format!("Unknown opcode: {:#x}", opcode)),
        }
    }

    pub fn is_halted(&self) -> bool {
        matches!(self.state, Status::Halted)
    }
}
