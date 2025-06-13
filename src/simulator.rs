use crate::ast::Instruction;
use crate::ast::{self, OwnedInstruction};

pub type Disassembly = Vec<OwnedInstruction>;
pub type Changes = Vec<AtomicChange>;
pub type Log = Vec<Changes>;

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
        }
    }

    /// Resets the simulator state to its initial values.
    pub fn reset(&mut self) {
        self.registers = [0; 13];
        self.instruction_pointer = 0;
        self.memory = [0; MEM_SIZE];
    }

    fn apply_changes(&mut self, changes: &Changes) {
        for change in changes {
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
    }
    /// Executes the given instruction until it halts
    pub fn run_single(&mut self) -> Option<(OwnedInstruction, Changes)> {
        let fetch_result = self.fetch_decode();
        if let Err(e) = fetch_result {
            self.state = Status::Error(e);
            return None;
        }
        let instruction = fetch_result.unwrap();
        let changes;
        match instruction {
            Instruction::Halt => {
                changes = vec![AtomicChange::State {
                    status: Status::Halted,
                }];
            }
            Instruction::Nop => {
                changes = vec![AtomicChange::InstructionPointer {
                    ip: self.instruction_pointer + 1,
                }];
            }
            // Handle other instructions...
            _ => todo!(),
        }
        self.apply_changes(&changes);
        Some((instruction, changes))
    }

    fn fetch_decode(&self) -> Result<OwnedInstruction, String> {
        let byte0 = self
            .source
            .get(self.instruction_pointer as usize)
            .ok_or_else(|| format!("Instruction Pointer of Range: {}", self.instruction_pointer))?;

        let opcode = byte0 >> 4;
        let _func = byte0 & 0x0F;

        match opcode {
            0x0 => Ok(Instruction::Halt),
            0x1 => Ok(Instruction::Nop),
            // Add more opcodes as needed
            _ => Err(format!("Unknown opcode: {:#x}", opcode)),
        }
    }
}
