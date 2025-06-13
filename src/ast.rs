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

#[derive(Debug, Clone, Copy, PartialEq)]
/// Represents a line in the assembly code.
pub enum Instruction<S> {
    Label(S),
    Directive(S, ImmType),
    Halt,
    Nop,
    Irmov(LabOrImm<S>, Register),
    Rmmov(Register, ImmType, Register),
    Mrmov(ImmType, Register, Register),
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