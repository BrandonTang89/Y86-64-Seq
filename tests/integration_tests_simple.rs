use y86_seq::assembler::parse_and_gen;
use y86_seq::ast::Register;
use y86_seq::simulator::simulate;

#[test]
/// Tests nop, rrmovq, and halt instructions
fn integration_tests_simple_1() {
    let src_asm = r#"
nop
nop
rrmovq %rax, %rbx
halt
        "#;

    let machine_code = parse_and_gen(src_asm)
        .unwrap_or_else(|e| panic!("Parsing failed: {:?}", e))
        .1
        .bytes;
    let simulator = simulate::<1024>(&machine_code);

    assert!(simulator.is_halted(), "Simulator did not halt as expected");
    assert_eq!(
        simulator.registers[Register::Rax as usize],
        0,
        "RAX should be 0 after nop instructions"
    );
    assert_eq!(
        simulator.registers[Register::Rbx as usize],
        0,
        "RBX should be 0 after rrmovq from RAX to RBX"
    );
    assert_eq!(
        simulator.instruction_pointer, 4,
        "Instruction pointer should be at 4 after halting on 4th instruction"
    );
}

#[test]
/// Tests irmovq, rrmovq, and rmmovq instructions
fn integration_tests_simple_2() {
    let src_asm = r#"
irmovq $5, %rax
rrmovq %rax, %rbx
rmmovq %rbx, 0(%rax)
halt
        "#;

    let machine_code = parse_and_gen(src_asm)
        .unwrap_or_else(|e| panic!("Parsing failed: {:?}", e))
        .1
        .bytes;
    let simulator = simulate::<1024>(&machine_code);

    assert!(simulator.is_halted(), "Simulator did not halt as expected");
    assert_eq!(
        simulator.registers[Register::Rax as usize],
        5,
        "RAX should be 5 after irmovq $5, %rax"
    );
    assert_eq!(
        simulator.registers[Register::Rbx as usize],
        5,
        "RBX should be 5 after rrmovq %rax, %rbx"
    );
    assert_eq!(
        simulator.memory[5], 5,
        "Memory at address 0(%rax) should be 5 after rmmovq
        %rbx, 0(%rax)"
    );
}

#[test]
/// Tests rmmovq, mrmovq, and halt instructions
fn integration_tests_simple_3() {
    let src_asm = r#"
irmovq $10, %rax
irmovq $20, %rbx
rmmovq %rax, 0(%rbx)
mrmovq 0(%rbx), %rcx
halt
        "#;
    let machine_code = parse_and_gen(src_asm)
        .unwrap_or_else(|e| panic!("Parsing failed: {:?}", e))
        .1
        .bytes;
    let simulator = simulate::<1024>(&machine_code);
    assert!(simulator.is_halted(), "Simulator did not halt as expected");
    assert_eq!(
        simulator.registers[Register::Rax as usize],
        10,
        "RAX should be 10 after irmovq $10, %rax"
    );
    assert_eq!(
        simulator.registers[Register::Rbx as usize],
        20,
        "RBX should be 20 after irmovq $20, %rbx"
    );
    assert_eq!(
        simulator.memory[20], 10,
        "Memory at address 0(%rbx) should be 10 after rmmov
        %rax, 0(%rbx)"
    );
    assert_eq!(
        simulator.registers[Register::Rcx as usize],
        10,
        "RCX should be 10 after mrmovq 0(%rbx),
        %rcx"
    );
}

#[test]
/// Tests call and ret instructions
fn integration_tests_simple_4() {
    let src_asm = r#"
irmovq $100, %rax
call function
irmovq $200, %rbx
halt

function:
irmovq $50, %rcx
ret
        "#;
    let machine_code = parse_and_gen(src_asm)
        .unwrap_or_else(|e| panic!("Parsing failed: {:?}", e))
        .1
        .bytes;
    let simulator = simulate::<1024>(&machine_code);

    assert!(simulator.is_halted(), "Simulator did not halt as expected");
    assert_eq!(
        simulator.registers[Register::Rax as usize],
        100,
        "RAX should be 100 after irmovq $100, %rax"
    );
    assert_eq!(
        simulator.registers[Register::Rbx as usize],
        200,
        "RBX should be 200 after returning from function call"
    );
    assert_eq!(
        simulator.registers[Register::Rcx as usize],
        50,
        "RCX should be 50 after function execution"
    );
}

#[test]
/// Tests binary operations: addq, subq, andq, xorq
fn integration_tests_simple_5() {
    let src_asm = r#"
irmovq $10, %rax
irmovq $5, %rbx
addq %rbx, %rax
irmovq $15, %rcx
irmovq $7, %rdx
subq %rdx, %rcx
irmovq $12, %rsi
irmovq $10, %rdi
andq %rdi, %rsi
irmovq $15, %r8
irmovq $10, %r9
xorq %r9, %r8
halt
        "#;

    let machine_code = parse_and_gen(src_asm)
        .unwrap_or_else(|e| panic!("Parsing failed: {:?}", e))
        .1
        .bytes;
    let simulator = simulate::<1024>(&machine_code);

    assert!(simulator.is_halted(), "Simulator did not halt as expected");
    assert_eq!(
        simulator.registers[Register::Rax as usize],
        15,
        "RAX should be 15 after addq %rbx, %rax (10 + 5)"
    );
    assert_eq!(
        simulator.registers[Register::Rcx as usize],
        8,
        "RCX should be 8 after subq %rdx, %rcx (15 - 7)"
    );
    assert_eq!(
        simulator.registers[Register::Rsi as usize],
        8,
        "RSI should be 8 after andq %rdi, %rsi (12 & 10)"
    );
    assert_eq!(
        simulator.registers[Register::R8 as usize],
        5,
        "R8 should be 5 after xorq %r9, %r8 (15 ^ 10)"
    );
}
