use chumsky::prelude::*;
use chumsky::text::ascii::keyword;
use log::info;
type ImmType = i64;

#[derive(Debug, Clone)]
enum Register {
    RAX,
    RBX,
    RCX,
    RDX,
    RDI,
    RSI,
    RSP,
    RBP,
}

#[derive(Debug, Clone)]
enum LabOrImm<'a> {
    Labelled(&'a str),
    Immediate(ImmType),
}

#[derive(Debug, Clone)]
enum BinaryOp {
    Add = 0,
    Sub = 1,
    And = 2,
    Xor = 3,
}

#[derive(Debug, Clone)]
enum CondOp {
    Uncon = 0,
    Le = 1,
    L = 2,
    E = 3,
    Ne = 4,
    Ge = 5,
    G = 6,
}

#[derive(Debug, Clone)]
enum AssemblyLine<'a> {
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

fn parser<'a>() -> impl Parser<'a, &'a str, Vec<AssemblyLine<'a>>> {
    let reg = just('%')
        .ignore_then(choice((
            keyword("rax").to(Register::RAX),
            keyword("rbx").to(Register::RBX),
            keyword("rcx").to(Register::RCX),
            keyword("rdx").to(Register::RDX),
            keyword("rdi").to(Register::RDI),
            keyword("rsi").to(Register::RSI),
            keyword("rsp").to(Register::RSP),
            keyword("rbp").to(Register::RBP),
        )))
        .padded();

    let imm = just('$')
        .ignore_then(choice((
            text::int(10).map(|s: &str| s.parse::<ImmType>().unwrap()),
            just("0x")
                .ignore_then(text::int(16))
                .map(|s: &str| i64::from_str_radix(s, 16).unwrap()),
        )))
        .padded();

    let lab_or_imm = choice((
        imm.map(LabOrImm::Immediate),
        text::ascii::ident().padded().map(LabOrImm::Labelled),
    ));

    let label = text::ascii::ident()
        .then_ignore(just(':'))
        .padded()
        .map(|s| AssemblyLine::Label(s));

    let directive = choice((just(".align"), just(".quad")))
        .then(text::int(10).padded())
        .map(|(dir, value): (&str, &str)| {
            let value: ImmType = value.parse().unwrap();
            AssemblyLine::Directive(dir, value)
        });

    let halt = keyword("halt").to(AssemblyLine::Halt);

    let nop = keyword("nop").to(AssemblyLine::Nop);

    let mrmov = keyword("mrmovq").ignore_then(
        choice((
            reg.clone()
                .delimited_by(just('('), just(')'))
                .map(|r| (0, r)),
            imm.clone()
                .then(reg.clone().delimited_by(just('('), just(')'))),
        ))
        .padded()
        .then(just(',').ignore_then(reg.clone()))
        .map(|((offset, base), dest)| AssemblyLine::Mrmov(offset, base, dest)),
    );

    let irmov = text::ascii::keyword("irmovq")
        .ignore_then(lab_or_imm.clone())
        .then(just(',').ignore_then(reg.clone()))
        .map(|(imm, reg)| AssemblyLine::Irmov(imm, reg));

    let binop = choice((
        keyword("addq").to(BinaryOp::Add),
        keyword("subq").to(BinaryOp::Sub),
        keyword("andq").to(BinaryOp::And),
        keyword("xorq").to(BinaryOp::Xor),
    ))
    .then(reg.clone())
    .then(just(',').ignore_then(reg.clone()))
    .map(|((op, src), dst)| AssemblyLine::Binop(op, src, dst));

    let jmp = choice((
        keyword("jmp").to(CondOp::Uncon),
        keyword("jle").to(CondOp::Le),
        keyword("jl").to(CondOp::L),
        keyword("je").to(CondOp::E),
        keyword("jne").to(CondOp::Ne),
        keyword("jge").to(CondOp::Ge),
        keyword("jg").to(CondOp::G),
    ))
    .then(lab_or_imm.clone())
    .map(|(op, addr)| AssemblyLine::Jmp(op, addr));

    let cmov = choice((
        keyword("rrmovq").to(CondOp::Uncon),
        keyword("cmovle").to(CondOp::Le),
        keyword("cmovl").to(CondOp::L),
        keyword("cmove").to(CondOp::E),
        keyword("cmovne").to(CondOp::Ne),
        keyword("cmovge").to(CondOp::Ge),
        keyword("cmovg").to(CondOp::G),
    ))
    .then(reg.clone())
    .then(reg.clone())
    .map(|((op, src), dst)| AssemblyLine::Cmov(op, src, dst));

    let call = keyword("call")
        .ignore_then(lab_or_imm.clone())
        .map(AssemblyLine::Call);

    let ret = keyword("ret").to(AssemblyLine::Ret);

    let push = keyword("pushq")
        .ignore_then(reg.clone())
        .map(AssemblyLine::Push);

    let pop = keyword("popq")
        .ignore_then(reg.clone())
        .map(AssemblyLine::Pop);

    let program = choice((
        label,
        directive,
        halt,
        nop,
        mrmov,
        irmov.clone(),
        binop,
        jmp,
        cmov,
        call,
        ret,
        push,
        pop,
    ))
    .padded()
    .repeated()
    .collect::<Vec<_>>();

    program
}

pub fn assemble(src: &str, highest_addr: usize) -> Result<Vec<u8>, String> {
    // Placeholder for the actual assembly logic
    // This function should parse the source code and generate machine code
    // For now, we will just return an empty vector to simulate successful assembly

    let memory: Vec<u8> = Vec::with_capacity(highest_addr);

    let parse_result = parser()
        .parse(src)
        .into_output()
        .expect("Failed to parse assembly code");

    eprintln!("Parsed assembly lines: {:?}", parse_result);
    for line in parse_result {
        info!("LINE: {:?}", line);
    }

    Ok(memory)
}

#[cfg(test)]
mod lib_tests;
