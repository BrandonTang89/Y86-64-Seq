pub use crate::ast::*;
use chumsky::prelude::*;
use chumsky::text::ascii::keyword;
mod parser_tests;

fn reg_parser<'a>() -> Boxed<'a, 'a, &'a str, Register, extra::Err<Simple<'a, char>>> {
    just('%')
        .ignore_then(choice((
            keyword("rax").to(Register::Rax),
            keyword("rbx").to(Register::Rbx),
            keyword("rcx").to(Register::Rcx),
            keyword("rdx").to(Register::Rdx),
            keyword("rdi").to(Register::Rdi),
            keyword("rsi").to(Register::Rsi),
            keyword("rsp").to(Register::Rsp),
            keyword("rbp").to(Register::Rbp),
            keyword("r8").to(Register::R8),
            keyword("r9").to(Register::R9),
            keyword("r10").to(Register::R10),
            keyword("r11").to(Register::R11),
            keyword("r12").to(Register::R12),
        )))
        .padded()
        .boxed()
}

/// Handles parsing positive and negative decimals and hexadecimals
fn imm_parser<'a>() -> Boxed<'a, 'a, &'a str, ImmType, extra::Err<Simple<'a, char>>> {
    let pos_imm = (choice((
        just("0x")
            .ignore_then(text::digits(16).to_slice())
            .map(|s: &str| ImmType::from_str_radix(s, 16).unwrap()),
        text::digits(16)
            .to_slice()
            .map(|s: &str| s.parse::<ImmType>().unwrap()),
    )))
    .padded();

    choice((just('-').then(pos_imm).map(|(_, imm)| -imm), pos_imm)).boxed()
}

/// Parses either D(reg) or reg
fn displaced_reg_parser<'a>() -> Boxed<'a, 'a, &'a str, (ImmType, Register), extra::Err<Simple<'a, char>>> {
    let reg = reg_parser();
    let imm = imm_parser();

    choice((
        reg.clone()
            .delimited_by(just('('), just(')'))
            .map(|r| (0, r)),
        imm.clone()
            .then(reg.clone().delimited_by(just('('), just(')'))),
    ))
    .padded()
    .boxed()
}

/// Constructs a parser for the Y86-64 assembly language
pub fn mk_parser<'a>() -> impl Parser<'a, &'a str, Vec<BorrowedInstruction<'a>>, extra::Err<Simple<'a, char>>> {
    let reg = reg_parser();

    let dollar_imm = just('$').ignore_then(imm_parser());
    let imm = imm_parser();

    let lab_or_imm = choice((
        dollar_imm.map(LabOrImm::Immediate),
        text::ascii::ident().map(LabOrImm::Labelled),
    ))
    .padded();

    let label = text::ascii::ident()
        .then_ignore(just(':'))
        .padded()
        .map(Instruction::Label);

    let directive = choice((just(".align"), just(".quad")))
        .padded()
        .then(imm.clone())
        .map(|(dir, imm)| Instruction::Directive(dir, imm));

    let halt = keyword("halt").to(Instruction::Halt);

    let nop = keyword("nop").to(Instruction::Nop);

    let irmov = text::ascii::keyword("irmovq")
        .ignore_then(lab_or_imm.clone())
        .then(just(',').ignore_then(reg.clone()))
        .map(|(imm, reg)| Instruction::Irmov(imm, reg));

    let rmmov = keyword("rmmovq")
        .ignore_then(reg.clone())
        .then(just(',').ignore_then(displaced_reg_parser()))
        .map(|(src, (offset, base))| Instruction::Rmmov(src, offset, base));

    let mrmov = keyword("mrmovq").ignore_then(
        displaced_reg_parser()
            .then(just(',').ignore_then(reg.clone()))
            .map(|((offset, base), dest)| Instruction::Mrmov(offset, base, dest)),
    );

    let binop = choice((
        keyword("addq").to(BinaryOp::Add),
        keyword("subq").to(BinaryOp::Sub),
        keyword("andq").to(BinaryOp::And),
        keyword("xorq").to(BinaryOp::Xor),
    ))
    .then(reg.clone())
    .then(just(',').ignore_then(reg.clone()))
    .map(|((op, src), dst)| Instruction::Binop(op, src, dst));

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
    .map(|(op, addr)| Instruction::Jmp(op, addr));

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
    .then(just(',').ignore_then(reg.clone()))
    .map(|((op, src), dst)| Instruction::Cmov(op, src, dst));

    let call = keyword("call")
        .ignore_then(lab_or_imm.clone())
        .map(Instruction::Call);

    let ret = keyword("ret").to(Instruction::Ret);

    let push = keyword("pushq")
        .ignore_then(reg.clone())
        .map(Instruction::Push);

    let pop = keyword("popq")
        .ignore_then(reg.clone())
        .map(Instruction::Pop);

    choice((
        label, directive, halt, nop, rmmov, irmov, mrmov, binop, jmp, cmov, call, ret, push, pop,
    ))
    .padded()
    .repeated()
    .collect::<Vec<_>>()
}
