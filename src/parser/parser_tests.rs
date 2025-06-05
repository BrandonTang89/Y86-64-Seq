#[cfg(test)]
use super::*;

#[test]
fn test_int_parsing<'a>() {
    let src = "42";
    let parsed = imm_parser()
        .parse(src)
        .into_output()
        .expect("Failed to parse int");
    assert_eq!(parsed, 42);
}

#[test]
fn test_hex_parsing() {
    let src = "0x1F";
    let parsed = imm_parser()
        .parse(src)
        .into_output()
        .expect("Failed to parse hex");
    assert_eq!(parsed, 31); // 0x1F in decimal
}

#[test]
fn test_parse_label() {
    let src = "start:";
    let parsed = mk_parser().parse(src).into_output().unwrap();
    assert_eq!(parsed.len(), 1);
    match &parsed[0] {
        AssemblyLine::Label(label) => assert_eq!(*label, "start"),
        _ => panic!("Expected label"),
    }
}

#[test]
fn test_parse_directive() {
    let src = ".align 8";
    let parsed = mk_parser().parse(src).into_output().unwrap();
    assert_eq!(parsed.len(), 1);
    match &parsed[0] {
        AssemblyLine::Directive(dir, val) => {
            assert_eq!(*dir, ".align");
            assert_eq!(*val, 8);
        }
        _ => panic!("Expected directive"),
    }
}

#[test]
fn test_parse_directive_hex() {
    let src = ".quad 0x1F";
    let parsed = mk_parser().parse(src).into_output().unwrap();
    assert_eq!(parsed.len(), 1);
    match &parsed[0] {
        AssemblyLine::Directive(dir, val) => {
            assert_eq!(*dir, ".quad");
            assert_eq!(*val, 31); // 0x1F in decimal
        }
        _ => panic!("Expected directive"),
    }
}

#[test]
fn test_parse_halt_nop() {
    let src = "halt\nnop";
    let parsed = mk_parser().parse(src).into_output().unwrap();
    assert_eq!(parsed.len(), 2);
    matches!(parsed[0], AssemblyLine::Halt);
    matches!(parsed[1], AssemblyLine::Nop);
}

#[test]
fn test_parse_rrmov() {
    let src = "rrmovq %rax, %rbx";
    let parsed = mk_parser().parse(src).into_output().unwrap();
    assert_eq!(parsed.len(), 1);
    match &parsed[0] {
        AssemblyLine::Rrmov(Register::Rax, Register::Rbx) => {}
        _ => panic!("Expected rrmovq"),
    }
}

#[test]
fn test_parse_rmmov() {
    let src = "rmmovq %rax, 8(%rbx)";
    let parsed = mk_parser().parse(src).into_output().unwrap();
    assert_eq!(parsed.len(), 1);
    match &parsed[0] {
        AssemblyLine::Rmmov(Register::Rax, 8, Register::Rbx) => {}
        _ => panic!("Expected rmmovq"),
    }
}

#[test]
fn test_parse_irmov() {
    let src = "irmovq $42, %rax";
    let parsed = mk_parser().parse(src).into_output().unwrap();
    assert_eq!(parsed.len(), 1);
    match &parsed[0] {
        AssemblyLine::Irmov(LabOrImm::Immediate(42), Register::Rax) => {}
        _ => panic!("Expected irmovq"),
    }
}

#[test]
fn test_parse_binop() {
    let src = "addq %rax, %rbx";
    let parsed = mk_parser().parse(src).into_output().unwrap();
    assert_eq!(parsed.len(), 1);
    match &parsed[0] {
        AssemblyLine::Binop(BinaryOp::Add, Register::Rax, Register::Rbx) => {}
        _ => panic!("Expected addq"),
    }
}

#[test]
fn test_parse_jmp() {
    let src = "jmp somewhere";
    let parsed = mk_parser().parse(src).into_output().unwrap();
    assert_eq!(parsed.len(), 1);
    match &parsed[0] {
        AssemblyLine::Jmp(CondOp::Uncon, LabOrImm::Labelled("somewhere")) => {}
        _ => panic!("Expected jmp"),
    }
}

#[test]
fn test_parse_mrmov_displace_reg() {
    let src = "mrmovq 8(%rbp), %rax";
    let parsed = mk_parser().parse(src).into_output().unwrap();
    assert_eq!(parsed.len(), 1);
    match &parsed[0] {
        AssemblyLine::Mrmov(8, Register::Rbp, Register::Rax) => {}
        _ => panic!("Expected mrmovq"),
    }
}

#[test]
fn test_parse_mrmov_negative_displace() {
    let src = "mrmovq -8(%rbp), %rax";
    let parsed = mk_parser().parse(src).into_output().unwrap();
    assert_eq!(parsed.len(), 1);
    match &parsed[0] {
        AssemblyLine::Mrmov(-8, Register::Rbp, Register::Rax) => {}
        _ => panic!("Expected mrmovq"),
    }
}

#[test]
fn test_parse_mrmov_reg() {
    let src = "mrmovq (%rbp), %rax";
    let parsed = mk_parser().parse(src).into_output().unwrap();
    assert_eq!(parsed.len(), 1);
    match &parsed[0] {
        AssemblyLine::Mrmov(0, Register::Rbp, Register::Rax) => {}
        _ => panic!("Expected mrmovq"),
    }
}

#[test]
fn test_parse_push_pop() {
    let src = "pushq %rbx\npopq %rcx";
    let parsed = mk_parser().parse(src).into_output().unwrap();
    assert_eq!(parsed.len(), 2);
    matches!(parsed[0], AssemblyLine::Push(Register::Rbx));
    matches!(parsed[1], AssemblyLine::Pop(Register::Rcx));
}

#[test]
fn test_parser_comprehensive() {
    let src = "
jmp main
.align 8
array:
.quad 0x0000000000000001
.quad 0x0000000000000002
.quad 0x0000000000000003
.quad 0x0000000000000004
main:
irmovq array, %rdi
irmovq $4, %rsi
call sum
ret
sum:
irmovq $8, %r8
irmovq $1, %r9
xorq %rax, %rax
andq %rsi, %rsi
jmp test
loop:
mrmovq (%rdi), %r10
addq %r10, %rax
addq %r8, %rdi
subq %r9, %rsi
test:
jne loop
ret
";
    assert!(mk_parser().parse(src).into_output().is_some());
}
