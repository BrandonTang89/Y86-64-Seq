use y86_seq::assembler::parse_and_gen;
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
        simulator.registers[0], 0,
        "RAX should be 0 after nop instructions"
    );
    assert_eq!(
        simulator.registers[1], 0,
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
        simulator.registers[0], 5,
        "RAX should be 5 after irmovq $5, %rax"
    );
    assert_eq!(
        simulator.registers[1], 5,
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
        simulator.registers[0], 10,
        "RAX should be 10 after irmovq $10, %rax"
    );
    assert_eq!(
        simulator.registers[1], 20,
        "RBX should be 20 after irmovq $20, %rbx"
    );
    assert_eq!(
        simulator.memory[20], 10,
        "Memory at address 0(%rbx) should be 10 after rmmov
        %rax, 0(%rbx)"
    );
    assert_eq!(
        simulator.registers[2], 10,
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
        simulator.registers[0], 100,
        "RAX should be 100 after irmovq $100, %rax"
    );
    assert_eq!(
        simulator.registers[1], 200,
        "RBX should be 200 after returning from function call"
    );
    assert_eq!(
        simulator.registers[2], 50,
        "RCX should be 50 after function execution"
    );
}