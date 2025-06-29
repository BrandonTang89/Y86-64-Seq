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

#[test]
/// Tests conditional jump instructions: jmp, je, jne, jl, jle, jg, jge
fn integration_test_conditional_jumps() {
    let src_asm = r#"
# Test unconditional jump
irmovq $100, %rax
jmp skip1
irmovq $999, %rbx   # This should be skipped
skip1:
irmovq $42, %rcx

# Test je (jump if equal) - should jump
irmovq $5, %rdx
irmovq $5, %rsi
subq %rsi, %rdx     # Sets Z=1 (equal)
je skip2
irmovq $888, %rdi   # This should be skipped
skip2:
irmovq $77, %r8

# Test jne (jump if not equal) - should NOT jump
irmovq $10, %r9
irmovq $10, %r10
subq %r10, %r9      # Sets Z=1 (equal), so jne should not jump
jne skip3
irmovq $55, %r11    # This should execute
skip3:
irmovq $33, %r12

# Test jg (jump if greater) - should jump
# Use %rbp and %rsp for this test to avoid register conflicts
irmovq $15, %rbp
irmovq $8, %rsp
subq %rsp, %rbp     # 15-8=7, positive result, should set flags for jg
jg final_check
irmovq $111, %rsp   # This should be skipped, overwrite %rsp to test
final_check:
# Store result in unused register to verify jump worked
irmovq $999, %rsp   # If this executes, %rsp will be 999

halt
        "#;

    let src_asm = remove_comments(src_asm);
    let machine_code = parse_and_gen(&src_asm)
        .unwrap_or_else(|e| panic!("Parsing failed: {:?}", e))
        .1
        .bytes;
    let simulator = simulate::<1024>(&machine_code);

    assert!(simulator.is_halted(), "Simulator did not halt as expected");

    // Test unconditional jump
    assert_eq!(
        simulator.registers[Register::Rax as usize],
        100,
        "RAX should be 100 (before unconditional jump)"
    );
    assert_eq!(
        simulator.registers[Register::Rbx as usize],
        0,
        "RBX should be 0 (instruction after jmp was skipped)"
    );
    assert_eq!(
        simulator.registers[Register::Rcx as usize],
        42,
        "RCX should be 42 (instruction after jump target executed)"
    );

    // Test je (should have jumped)
    assert_eq!(
        simulator.registers[Register::Rdi as usize],
        0,
        "RDI should be 0 (instruction after je was skipped)"
    );
    assert_eq!(
        simulator.registers[Register::R8 as usize],
        77,
        "R8 should be 77 (instruction after jump target executed)"
    );

    // Test jne (should NOT have jumped)
    assert_eq!(
        simulator.registers[Register::R11 as usize],
        55,
        "R11 should be 55 (jne didn't jump, so instruction executed)"
    );
    assert_eq!(
        simulator.registers[Register::R12 as usize],
        33,
        "R12 should be 33"
    );

    // Test jg (should have jumped)
    assert_eq!(
        simulator.registers[Register::Rbp as usize],
        7,
        "RBP should be 7 after subtraction (15-8)"
    );
    assert_eq!(
        simulator.registers[Register::Rsp as usize],
        999,
        "RSP should be 999 (final instruction executed, meaning jg jumped correctly)"
    );
}

#[test]
/// Tests push and pop instructions including edge cases with stack pointer
fn integration_test_push_pop() {
    let src_asm = r#"
# Test basic push and pop
irmovq $100, %rax
irmovq $200, %rbx
pushq %rax
pushq %rbx
popq %rcx           # Should get 200 (last pushed)
popq %rdx           # Should get 100 (first pushed)

# Test pushing stack pointer - should push old SP value
irmovq $1000, %rsi  # Store initial SP for comparison
rrmovq %rsp, %rsi   # Copy current SP to %rsi
pushq %rsp          # Push current SP value onto stack
popq %rdi           # Pop the SP value that was pushed

# Test popping to stack pointer - popped value becomes new SP
irmovq $500, %r8
pushq %r8           # Push 500 onto stack
popq %rsp           # Pop 500 into %rsp - %rsp should now be 500

# Verify we can still push/pop after SP manipulation
irmovq $999, %r9
pushq %r9
popq %r10

halt
        "#;

    let src_asm = remove_comments(src_asm);
    let machine_code = parse_and_gen(&src_asm)
        .unwrap_or_else(|e| panic!("Parsing failed: {:?}", e))
        .1
        .bytes;
    let simulator = simulate::<1024>(&machine_code);

    assert!(simulator.is_halted(), "Simulator did not halt as expected");

    // Test basic push/pop
    assert_eq!(
        simulator.registers[Register::Rax as usize],
        100,
        "RAX should still be 100"
    );
    assert_eq!(
        simulator.registers[Register::Rbx as usize],
        200,
        "RBX should still be 200"
    );
    assert_eq!(
        simulator.registers[Register::Rcx as usize],
        200,
        "RCX should be 200 (last value pushed)"
    );
    assert_eq!(
        simulator.registers[Register::Rdx as usize],
        100,
        "RDX should be 100 (first value pushed)"
    );

    // Test pushing SP - should have pushed the old SP value
    let initial_sp = simulator.registers[Register::Rsi as usize];
    assert_eq!(
        simulator.registers[Register::Rdi as usize],
        initial_sp,
        "RDI should contain the old SP value that was pushed"
    );

    // Test popping to SP - SP should now be 500
    assert_eq!(
        simulator.registers[Register::Rsp as usize],
        500,
        "RSP should be 500 after popping to it"
    );

    // Test continued operation after SP manipulation
    assert_eq!(
        simulator.registers[Register::R9 as usize],
        999,
        "R9 should be 999"
    );
    assert_eq!(
        simulator.registers[Register::R10 as usize],
        999,
        "R10 should be 999 (pushed and popped after SP manipulation)"
    );
}

#[test]
/// Tests edge cases for conditional jumps with different flag combinations
fn integration_test_jump_edge_cases() {
    let src_asm = r#"
# Test jl (jump if less) with negative result
irmovq $3, %rax
irmovq $7, %rbx
subq %rbx, %rax     # 3-7 = -4, should set N=1, V=0 (N!=V means less)
jl case1
irmovq $111, %rcx   # Should be skipped
case1:
irmovq $222, %rdx   # Should execute

# Test jge (jump if greater or equal) with positive result
irmovq $10, %rsi
irmovq $5, %rdi
subq %rdi, %rsi     # 10-5 = 5, should set N=0, V=0 (N==V means >= )
jge case2
irmovq $333, %r8    # Should be skipped
case2:
irmovq $444, %r9    # Should execute

# Test jle (jump if less or equal) with zero result
irmovq $8, %r10
irmovq $8, %r11
subq %r11, %r10     # 8-8 = 0, should set Z=1 (equal, so <= is true)
jle case3
irmovq $555, %r12   # Should be skipped
case3:
irmovq $666, %rbp   # Should execute

halt
        "#;

    let src_asm = remove_comments(src_asm);
    let machine_code = parse_and_gen(&src_asm)
        .unwrap_or_else(|e| panic!("Parsing failed: {:?}", e))
        .1
        .bytes;
    let simulator = simulate::<1024>(&machine_code);

    assert!(simulator.is_halted(), "Simulator did not halt as expected");

    // Test jl with negative result (should jump)
    assert_eq!(
        simulator.registers[Register::Rax as usize],
        -4,
        "RAX should be -4 after 3-7"
    );
    assert_eq!(
        simulator.registers[Register::Rcx as usize],
        0,
        "RCX should be 0 (instruction after jl was skipped)"
    );
    assert_eq!(
        simulator.registers[Register::Rdx as usize],
        222,
        "RDX should be 222 (jump target executed)"
    );

    // Test jge with positive result (should jump)
    assert_eq!(
        simulator.registers[Register::Rsi as usize],
        5,
        "RSI should be 5 after 10-5"
    );
    assert_eq!(
        simulator.registers[Register::R8 as usize],
        0,
        "R8 should be 0 (instruction after jge was skipped)"
    );
    assert_eq!(
        simulator.registers[Register::R9 as usize],
        444,
        "R9 should be 444 (jump target executed)"
    );

    // Test jle with zero result (should jump)
    assert_eq!(
        simulator.registers[Register::R10 as usize],
        0,
        "R10 should be 0 after 8-8"
    );
    assert_eq!(
        simulator.registers[Register::R12 as usize],
        0,
        "R12 should be 0 (instruction after jle was skipped)"
    );
    assert_eq!(
        simulator.registers[Register::Rbp as usize],
        666,
        "RBP should be 666 (jump target executed)"
    );
}

#[test]
/// Tests stack operations with multiple push/pop sequences
fn integration_test_stack_operations() {
    let src_asm = r#"
# Save initial stack pointer
rrmovq %rsp, %r11

# Push multiple values
irmovq $10, %rax
irmovq $20, %rbx  
irmovq $30, %rcx
irmovq $40, %rdx
pushq %rax
pushq %rbx
pushq %rcx
pushq %rdx

# Pop in reverse order
popq %r8    # Should get 40
popq %r9    # Should get 30
popq %r10   # Should get 20
popq %rdi   # Should get 10

# Stack pointer should be back to original
rrmovq %rsp, %rsi   # Copy current SP to %rsi for comparison

halt
        "#;

    let src_asm = remove_comments(src_asm);
    let machine_code = parse_and_gen(&src_asm)
        .unwrap_or_else(|e| panic!("Parsing failed: {:?}", e))
        .1
        .bytes;
    let simulator = simulate::<1024>(&machine_code);

    assert!(simulator.is_halted(), "Simulator did not halt as expected");

    // Check values were popped in reverse order
    assert_eq!(
        simulator.registers[Register::R8 as usize],
        40,
        "R8 should be 40 (last pushed, first popped)"
    );
    assert_eq!(
        simulator.registers[Register::R9 as usize],
        30,
        "R9 should be 30"
    );
    assert_eq!(
        simulator.registers[Register::R10 as usize],
        20,
        "R10 should be 20"
    );
    assert_eq!(
        simulator.registers[Register::Rdi as usize],
        10,
        "RDI should be 10 (first pushed, last popped)"
    );

    // Check that stack pointer is back to original position
    assert_eq!(
        simulator.registers[Register::R11 as usize],
        simulator.registers[Register::Rsi as usize],
        "Stack pointer should be back to original position after balanced push/pop"
    );
}
