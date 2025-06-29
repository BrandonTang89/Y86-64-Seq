use y86_seq::assembler::{parse_and_gen, remove_comments};
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

// ...existing code...

#[test]
/// Tests conditional move operations: cmove, cmovne, cmovl, cmovle, cmovg, cmovge
fn integration_tests_simple_6() {
    let src_asm = r#"
# Test cmove (move if equal)
irmovq $10, %rax
irmovq $10, %rbx
subq %rbx, %rax         # This sets Z=1 (zero flag)
irmovq $42, %rcx
cmove %rcx, %rdx        # Should move 42 to %rdx since Z=1

# Test cmovne (move if not equal)
irmovq $5, %rax
irmovq $3, %rbx
subq %rbx, %rax         # This sets Z=0 (not zero)
irmovq $99, %rcx
cmovne %rcx, %rsi       # Should move 99 to %rsi since Z=0

# Test cmovl (move if less than)
irmovq $3, %rax
irmovq $7, %rbx
subq %rbx, %rax         # 3-7 = -4, sets N=1, V=0 (N!=V means less than)
irmovq $123, %rcx
cmovl %rcx, %rdi        # Should move 123 to %rdi since N!=V

# Test cmovge (move if greater or equal)
irmovq $7, %rax
irmovq $3, %rbx
subq %rbx, %rax         # 7-3 = 4, sets N=0, V=0 (N==V means greater or equal)
irmovq $456, %rcx
cmovge %rcx, %r8        # Should move 456 to %r8 since N==V

# Test cmovg (move if greater)
irmovq $10, %rax
irmovq $5, %rbx
subq %rbx, %rax         # 10-5 = 5, sets Z=0, N=0, V=0 (Z==0 && N==V means greater)
irmovq $789, %rcx
cmovg %rcx, %r9         # Should move 789 to %r9 since Z==0 && N==V

# Test cmovle (move if less or equal) - should NOT move
irmovq $8, %rax
irmovq $2, %rbx
subq %rbx, %rax         # 8-2 = 6, sets Z=0, N=0, V=0 (Z==1 || N!=V is false)
irmovq $999, %rcx
cmovle %rcx, %r10       # Should NOT move since condition is false

halt
        "#;

    let src_asm = remove_comments(src_asm);
    let machine_code = parse_and_gen(&src_asm)
        .unwrap_or_else(|e| panic!("Parsing failed: {:?}", e))
        .1
        .bytes;
    let simulator = simulate::<1024>(&machine_code);

    assert!(simulator.is_halted(), "Simulator did not halt as expected");

    // Test cmove (should have moved)
    assert_eq!(
        simulator.registers[Register::Rdx as usize],
        42,
        "RDX should be 42 after cmove when Z=1"
    );

    // Test cmovne (should have moved)
    assert_eq!(
        simulator.registers[Register::Rsi as usize],
        99,
        "RSI should be 99 after cmovne when Z=0"
    );

    // Test cmovl (should have moved)
    assert_eq!(
        simulator.registers[Register::Rdi as usize],
        123,
        "RDI should be 123 after cmovl when N!=V"
    );

    // Test cmovge (should have moved)
    assert_eq!(
        simulator.registers[Register::R8 as usize],
        456,
        "R8 should be 456 after cmovge when N==V"
    );

    // Test cmovg (should have moved)
    assert_eq!(
        simulator.registers[Register::R9 as usize],
        789,
        "R9 should be 789 after cmovg when Z==0 && N==V"
    );

    // Test cmovle (should NOT have moved)
    assert_eq!(
        simulator.registers[Register::R10 as usize],
        0,
        "R10 should remain 0 after cmovle when condition is false"
    );
}
// ...existing code...
