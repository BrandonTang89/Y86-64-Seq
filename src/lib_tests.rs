#[cfg(test)]
use super::*;

#[test]
fn test_parse_label() {
    let src = "start:";
    let parsed = parser().parse(src).into_output().unwrap();
    assert_eq!(parsed.len(), 1);
    match &parsed[0] {
        AssemblyLine::Label(label) => assert_eq!(*label, "start"),
        _ => panic!("Expected label"),
    }
}

#[test]
fn test_parse_directive() {
    let src = ".align 8";
    let parsed = parser().parse(src).into_output().unwrap();
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
fn test_parse_halt_nop() {
    let src = "halt\nnop";
    let parsed = parser().parse(src).into_output().unwrap();
    assert_eq!(parsed.len(), 2);
    matches!(parsed[0], AssemblyLine::Halt);
    matches!(parsed[1], AssemblyLine::Nop);
}

#[test]
fn test_parse_irmov() {
    let src = "irmovq $42, %rax";
    let parsed = parser().parse(src).into_output().unwrap();
    assert_eq!(parsed.len(), 1);
    match &parsed[0] {
        AssemblyLine::Irmov(LabOrImm::Immediate(42), Register::RAX) => {}
        _ => panic!("Expected irmovq"),
    }
}

#[test]
fn test_parse_binop() {
    let src = "addq %rax, %rbx";
    let parsed = parser().parse(src).into_output().unwrap();
    assert_eq!(parsed.len(), 1);
    match &parsed[0] {
        AssemblyLine::Binop(BinaryOp::Add, Register::RAX, Register::RBX) => {}
        _ => panic!("Expected addq"),
    }
}

#[test]
fn test_parse_jmp() {
    let src = "jmp somewhere";
    let parsed = parser().parse(src).into_output().unwrap();
    assert_eq!(parsed.len(), 1);
    match &parsed[0] {
        AssemblyLine::Jmp(CondOp::Uncon, LabOrImm::Labelled("somewhere")) => {}
        _ => panic!("Expected jmp"),
    }
}

#[test]
fn test_parse_mrmov_displace_reg() {
    let src = "mrmovq $8(%rbp), %rax";
    let parsed = parser().parse(src).into_output().unwrap();
    assert_eq!(parsed.len(), 1);
    match &parsed[0] {
        AssemblyLine::Mrmov(8, Register::RBP, Register::RAX) => {}
        _ => panic!("Expected mrmovq"),
    }
}

#[test]
fn test_parse_mrmov_reg() {
    let src = "mrmovq (%rbp), %rax";
    let parsed = parser().parse(src).into_output().unwrap();
    assert_eq!(parsed.len(), 1);
    match &parsed[0] {
        AssemblyLine::Mrmov(0, Register::RBP, Register::RAX) => {}
        _ => panic!("Expected mrmovq"),
    }
}

#[test]
fn test_parse_push_pop() {
    let src = "pushq %rbx\npopq %rcx";
    let parsed = parser().parse(src).into_output().unwrap();
    assert_eq!(parsed.len(), 2);
    matches!(parsed[0], AssemblyLine::Push(Register::RBX));
    matches!(parsed[1], AssemblyLine::Pop(Register::RCX));
}
