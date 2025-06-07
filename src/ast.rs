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

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LabOrImm<'a> {
    Labelled(&'a str),
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

#[derive(Debug, Clone, Copy, PartialEq)]
/// Represents a line in the assembly code.
pub enum AssemblyLine<'a> {
    Label(&'a str),
    Directive(&'a str, ImmType),
    Halt,
    Nop,
    Irmov(LabOrImm<'a>, Register),
    Rmmov(Register, ImmType, Register),
    Mrmov(ImmType, Register, Register),
    Binop(BinaryOp, Register, Register),
    Jmp(CondOp, LabOrImm<'a>),
    Cmov(CondOp, Register, Register),
    Call(LabOrImm<'a>),
    Ret,
    Push(Register),
    Pop(Register),
}
