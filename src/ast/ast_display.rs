use super::*;

use std::fmt;

impl fmt::Display for Register {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let reg_name = match self {
            Register::Rax => "rax",
            Register::Rbx => "rbx",
            Register::Rcx => "rcx",
            Register::Rdx => "rdx",
            Register::Rdi => "rdi",
            Register::Rsi => "rsi",
            Register::Rsp => "rsp",
            Register::Rbp => "rbp",
            Register::R8 => "r8",
            Register::R9 => "r9",
            Register::R10 => "r10",
            Register::R11 => "r11",
            Register::R12 => "r12",
        };
        write!(f, "{}", reg_name)
    }
}

impl<S: fmt::Display> fmt::Display for LabOrImm<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LabOrImm::Labelled(label) => write!(f, "{}", label),
            LabOrImm::Immediate(imm) => write!(f, "{}", imm),
        }
    }
}

impl fmt::Display for BinaryOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let op = match self {
            BinaryOp::Add => "add",
            BinaryOp::Sub => "sub",
            BinaryOp::And => "and",
            BinaryOp::Xor => "xor",
        };
        write!(f, "{}", op)
    }
}

impl fmt::Display for CondOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let op = match self {
            CondOp::Uncon => "uncon",
            CondOp::Le => "le",
            CondOp::Lt => "l",
            CondOp::Eq => "e",
            CondOp::Ne => "ne",
            CondOp::Ge => "ge",
            CondOp::Gt => "g",
        };
        write!(f, "{}", op)
    }
}

impl<S: fmt::Display> fmt::Display for Instruction<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Instruction::Label(_) => {}
            _ => {
                write!(f, "    ")?; // Indent assembly lines
            }
        }
        match self {
            Instruction::Label(label) => write!(f, "{}:", label),
            Instruction::Directive(directive, imm) => write!(f, "{} {}", directive, imm),
            Instruction::Halt => write!(f, "halt"),
            Instruction::Nop => write!(f, "nop"),
            Instruction::Irmov(lab_or_imm, reg) => write!(f, "irmov {}, {}", lab_or_imm, reg),
            Instruction::Rmmov(src_reg, imm, dest_reg) => {
                write!(f, "rmmov {}, {}({})", src_reg, imm, dest_reg)
            }
            Instruction::Mrmov(imm, src_reg, dest_reg) => {
                write!(f, "mrmov {}({}) {}", imm, src_reg, dest_reg)
            }
            Instruction::Binop(op, src_reg, dest_reg) => {
                write!(f, "{} {}, {}", op, src_reg, dest_reg)
            }
            Instruction::Jmp(cond_op, lab_or_imm) => {
                if cond_op == &CondOp::Uncon {
                    write!(f, "jmp {}", lab_or_imm)
                } else {
                    write!(f, "j{} {}", cond_op, lab_or_imm)
                }
            }
            Instruction::Cmov(cond_op, src_reg, dest_reg) => {
                if cond_op == &CondOp::Uncon {
                    write!(f, "mov {}, {}", src_reg, dest_reg)
                } else {
                    write!(f, "cmov_{} {}, {}", cond_op, src_reg, dest_reg)
                }
            }
            Instruction::Call(lab_or_imm) => write!(f, "call {}", lab_or_imm),
            Instruction::Ret => write!(f, "ret"),
            Instruction::Push(reg) => write!(f, "push {}", reg),
            Instruction::Pop(reg) => write!(f, "pop {}", reg),
        }
    }
}
