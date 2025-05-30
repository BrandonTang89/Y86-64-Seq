pub use crate::ast::*;
use chumsky::prelude::*;
use chumsky::text::ascii::keyword;
mod parser_tests;

fn imm_parser<'a>() -> Boxed<'a, 'a, &'a str, ImmType> {
    // Handles parsing positive and negative decimals and hexadecimals
    let pos_imm = (choice((
        just("0x")
            .ignore_then(text::digits(16).to_slice())
            .map(|s: &str| ImmType::from_str_radix(s, 16).unwrap()),
        text::digits(16)
            .to_slice()
            .map(|s: &str| s.parse::<ImmType>().unwrap()),
    )))
    .padded();

    choice((
        just('-').then(pos_imm.clone()).map(|(_, imm)| -imm),
        pos_imm,
    ))
    .boxed()
}

pub fn mk_parser<'a>() -> impl Parser<'a, &'a str, Vec<AssemblyLine<'a>>> {
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
            keyword("r8").to(Register::R8),
            keyword("r9").to(Register::R9),
            keyword("r10").to(Register::R10),
            keyword("r11").to(Register::R11),
            keyword("r12").to(Register::R12),
        )))
        .padded();

    let dollar_imm = just('$').ignore_then(imm_parser());
    let imm = imm_parser();

    let lab_or_imm = choice((
        dollar_imm.clone().map(LabOrImm::Immediate),
        text::ascii::ident().map(LabOrImm::Labelled),
    ))
    .padded();

    let label = text::ascii::ident()
        .then_ignore(just(':'))
        .padded()
        .map(|s| AssemblyLine::Label(s));

    let directive = choice((just(".align"), just(".quad")))
        .then(imm.clone())
        .map(|(dir, imm)| AssemblyLine::Directive(dir, imm));

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
        label, directive, halt, nop, mrmov, irmov, binop, jmp, cmov, call, ret, push, pop,
    ))
    .padded()
    .repeated()
    .collect::<Vec<_>>();

    program
}
