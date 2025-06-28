mod ast_display;

pub type ImmType = i64;
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Register {
    Rax = 0,
    Rbx = 1,
    Rcx = 2,
    Rdx = 3,
    Rdi = 4,
    Rsi = 5,
    Rsp = 6,
    Rbp = 7,
    R8 = 8,
    R9 = 9,
    R10 = 10,
    R11 = 11,
    R12 = 12,
}

impl TryFrom<u8> for Register {
    type Error = String;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Register::Rax),
            1 => Ok(Register::Rbx),
            2 => Ok(Register::Rcx),
            3 => Ok(Register::Rdx),
            4 => Ok(Register::Rdi),
            5 => Ok(Register::Rsi),
            6 => Ok(Register::Rsp),
            7 => Ok(Register::Rbp),
            8 => Ok(Register::R8),
            9 => Ok(Register::R9),
            10 => Ok(Register::R10),
            11 => Ok(Register::R11),
            12 => Ok(Register::R12),
            _ => Err(format!("Invalid register value: {}", value)),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum LabOrImm<S> {
    Labelled(S),
    Immediate(ImmType),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BinaryOp {
    Add = 0,
    Sub = 1,
    And = 2,
    Xor = 3,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CondOp {
    Uncon = 0,
    Le = 1,
    L = 2,
    E = 3,
    Ne = 4,
    Ge = 5,
    G = 6,
}

#[derive(Debug, Clone, PartialEq)]
/// Represents a line in the assembly code.
pub enum Instruction<S> {
    Label(S),
    Directive(S, ImmType),
    Halt,
    Nop,

    /// Source, Destination
    Irmov(LabOrImm<S>, Register),

    /// Source, Displacement, Destination
    Rmmov(Register, ImmType, Register),

    /// Displacement, Source, Destination
    Mrmov(ImmType, Register, Register),

    /// Operator, Source, Destination
    Binop(BinaryOp, Register, Register),

    Jmp(CondOp, LabOrImm<S>),
    Cmov(CondOp, Register, Register),
    Call(LabOrImm<S>),
    Ret,
    Push(Register),
    Pop(Register),
}

pub type BorrowedInstruction<'a> = Instruction<&'a str>;
pub type OwnedInstruction = Instruction<String>;
