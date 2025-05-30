pub type ImmType = i64;
#[derive(Debug, Clone)]
pub enum Register {
    RAX,
    RBX,
    RCX,
    RDX,
    RDI,
    RSI,
    RSP,
    RBP,
    R8,
    R9,
    R10,
    R11,
    R12,
}

#[derive(Debug, Clone)]
pub enum LabOrImm<'a> {
    Labelled(&'a str),
    Immediate(ImmType),
}

#[derive(Debug, Clone)]
pub enum BinaryOp {
    Add = 0,
    Sub = 1,
    And = 2,
    Xor = 3,
}

#[derive(Debug, Clone)]
pub enum CondOp {
    Uncon = 0,
    Le = 1,
    L = 2,
    E = 3,
    Ne = 4,
    Ge = 5,
    G = 6,
}

#[derive(Debug, Clone)]
pub enum AssemblyLine<'a> {
    Label(&'a str),
    Directive(&'a str, ImmType),
    Halt,
    Nop,
    Irmov(LabOrImm<'a>, Register),
    Mrmov(ImmType, Register, Register),
    Binop(BinaryOp, Register, Register),
    Jmp(CondOp, LabOrImm<'a>),
    Cmov(CondOp, Register, Register),
    Call(LabOrImm<'a>),
    Ret,
    Push(Register),
    Pop(Register),
}
