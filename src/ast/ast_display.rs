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

impl fmt::Display for LabOrImm<'_> {
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
            CondOp::L => "l",
            CondOp::E => "e",
            CondOp::Ne => "ne",
            CondOp::Ge => "ge",
            CondOp::G => "g",
        };
        write!(f, "{}", op)
    }
}

impl fmt::Display for AssemblyLine<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AssemblyLine::Label(_) => {}
            _ => {
                write!(f, "    ")?; // Indent assembly lines
            }
        }
        match self {
            AssemblyLine::Label(label) => write!(f, "{}:", label),
            AssemblyLine::Directive(directive, imm) => write!(f, "{} {}", directive, imm),
            AssemblyLine::Halt => write!(f, "halt"),
            AssemblyLine::Nop => write!(f, "nop"),
            AssemblyLine::Irmov(lab_or_imm, reg) => write!(f, "irmov {}, {}", lab_or_imm, reg),
            AssemblyLine::Rmmov(src_reg, imm, dest_reg) => {
                write!(f, "rmmov {}, {}({})", src_reg, imm, dest_reg)
            }
            AssemblyLine::Mrmov(imm, src_reg, dest_reg) => {
                write!(f, "mrmov {}({}) {}", imm, src_reg, dest_reg)
            }
            AssemblyLine::Binop(op, src_reg, dest_reg) => {
                write!(f, "{} {}, {}", op, src_reg, dest_reg)
            }
            AssemblyLine::Jmp(cond_op, lab_or_imm) => {
                if cond_op == &CondOp::Uncon {
                    write!(f, "jmp {}", lab_or_imm)
                } else {
                    write!(f, "j{} {}", cond_op, lab_or_imm)
                }
            }
            AssemblyLine::Cmov(cond_op, src_reg, dest_reg) => {
                if cond_op == &CondOp::Uncon {
                    write!(f, "mov {}, {}", src_reg, dest_reg)
                } else {
                    write!(f, "cmov_{} {}, {}", cond_op, src_reg, dest_reg)
                }
            }
            AssemblyLine::Call(lab_or_imm) => write!(f, "call {}", lab_or_imm),
            AssemblyLine::Ret => write!(f, "ret"),
            AssemblyLine::Push(reg) => write!(f, "push {}", reg),
            AssemblyLine::Pop(reg) => write!(f, "pop {}", reg),
        }
    }
}
