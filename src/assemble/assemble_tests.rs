#[cfg(test)]
use super::*;
use crate::ast::AssemblyLine;
use crate::parser::mk_parser;
use chumsky::prelude::*;

#[test]
fn test_code_gen_quad() {
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

    let assembled_code = gen_code(&parsed).unwrap();
    assert_eq!(assembled_code.bytes.len(), 8);
    assert_eq!(assembled_code.bytes[0..8], [31, 0, 0, 0, 0, 0, 0, 0]);
}

#[test]
fn test_code_gen_halt() {
    let src = "halt";
    let parsed = mk_parser().parse(src).into_output().unwrap();
    let assembled_code = gen_code(&parsed).unwrap();
    assert_eq!(assembled_code.bytes.len(), 1);
    assert_eq!(assembled_code.bytes[0], 0x00); // HALT opcode
}

#[test]
fn test_code_gen_nop() {
    let src = "nop";
    let parsed = mk_parser().parse(src).into_output().unwrap();
    let assembled_code = gen_code(&parsed).unwrap();
    assert_eq!(assembled_code.bytes.len(), 1);
    assert_eq!(assembled_code.bytes[0], 0x10); // NOP opcode
}

#[test]
fn test_code_gen_irmov() {
    let src = "irmovq $42, %rax";
    let parsed = mk_parser().parse(src).into_output().unwrap();
    let assembled_code = gen_code(&parsed).unwrap();
    assert_eq!(assembled_code.bytes.len(), 10);
    assert_eq!(assembled_code.bytes[0], 0x30); // IRMOV opcode
    // Immediate value 42 in little-endian
    assert_eq!(assembled_code.bytes[2..10], [42, 0, 0, 0, 0, 0, 0, 0]);
}

#[test]
fn test_code_gen_rmmov() {
    let src = "rmmovq %rax, 8(%rbx)";
    let parsed = mk_parser().parse(src).into_output().unwrap();
    let assembled_code = gen_code(&parsed).unwrap();
    assert_eq!(assembled_code.bytes.len(), 10);
    assert_eq!(assembled_code.bytes[0], 0x40); // RMMOV opcode
    assert_eq!(assembled_code.bytes[1], 0x01); // rax=0, rbx=1
    // Displacement 8 in little-endian
    assert_eq!(assembled_code.bytes[2..10], [8, 0, 0, 0, 0, 0, 0, 0]);
}

#[test]
fn test_code_gen_mrmov() {
    let src = "mrmovq 8(%rbp), %rax";
    let parsed = mk_parser().parse(src).into_output().unwrap();
    let assembled_code = gen_code(&parsed).unwrap();
    assert_eq!(assembled_code.bytes.len(), 10);
    assert_eq!(assembled_code.bytes[0], 0x50); // MRMOV opcode
    assert_eq!(assembled_code.bytes[1], 0x07); // rbp=7, rax=0
    // Displacement 8 in little-endian
    assert_eq!(assembled_code.bytes[2..10], [8, 0, 0, 0, 0, 0, 0, 0]);
}

#[test]
fn test_code_gen_mrmov_negative_displacement() {
    let src = "mrmovq -8(%rbp), %rax";
    let parsed = mk_parser().parse(src).into_output().unwrap();
    let assembled_code = gen_code(&parsed).unwrap();
    assert_eq!(assembled_code.bytes.len(), 10);
    assert_eq!(assembled_code.bytes[0], 0x50); // MRMOV opcode
    assert_eq!(assembled_code.bytes[1], 0x07); // rbp=7, rax=0
    // Displacement -8 in little-endian (two's complement)
    assert_eq!(
        assembled_code.bytes[2..10],
        [248, 255, 255, 255, 255, 255, 255, 255]
    );
}

#[test]
fn test_code_gen_binop_add() {
    let src = "addq %rax, %rbx";
    let parsed = mk_parser().parse(src).into_output().unwrap();
    let assembled_code = gen_code(&parsed).unwrap();
    assert_eq!(assembled_code.bytes.len(), 2);
    assert_eq!(assembled_code.bytes[0], 0x60); // ADD opcode
    assert_eq!(assembled_code.bytes[1], 0x01); // rax=0, rbx=1
}

#[test]
fn test_code_gen_binop_sub() {
    let src = "subq %rdi, %rsi";
    let parsed = mk_parser().parse(src).into_output().unwrap();
    let assembled_code = gen_code(&parsed).unwrap();
    assert_eq!(assembled_code.bytes.len(), 2);
    assert_eq!(assembled_code.bytes[0], 0x61); // SUB opcode
    assert_eq!(assembled_code.bytes[1], 0x45); // rdi=4, rsi=5
}

#[test]
fn test_code_gen_binop_and() {
    let src = "andq %rdx, %rcx";
    let parsed = mk_parser().parse(src).into_output().unwrap();
    let assembled_code = gen_code(&parsed).unwrap();
    assert_eq!(assembled_code.bytes.len(), 2);
    assert_eq!(assembled_code.bytes[0], 0x62); // AND opcode
    assert_eq!(assembled_code.bytes[1], 0x32); // rdx=3, rcx=2
}

#[test]
fn test_code_gen_binop_xor() {
    let src = "xorq %r8, %r9";
    let parsed = mk_parser().parse(src).into_output().unwrap();
    let assembled_code = gen_code(&parsed).unwrap();
    assert_eq!(assembled_code.bytes.len(), 2);
    assert_eq!(assembled_code.bytes[0], 0x63); // XOR opcode
    assert_eq!(assembled_code.bytes[1], 0x89); // r8=8, r9=9
}

#[test]
fn test_code_gen_jmp() {
    let src = "jmp main\nmain:";
    let parsed = mk_parser().parse(src).into_output().unwrap();
    let assembled_code = gen_code(&parsed).unwrap();
    assert_eq!(assembled_code.bytes.len(), 9);
    assert_eq!(assembled_code.bytes[0], 0x70); // JMP opcode
    // Target address 9 (after the jmp instruction) in little-endian
    assert_eq!(assembled_code.bytes[1..9], [9, 0, 0, 0, 0, 0, 0, 0]);
}

#[test]
fn test_code_gen_jl() {
    let src = "jl loop\nloop:";
    let parsed = mk_parser().parse(src).into_output().unwrap();
    let assembled_code = gen_code(&parsed).unwrap();
    assert_eq!(assembled_code.bytes.len(), 9);
    assert_eq!(assembled_code.bytes[0], 0x72); // JL opcode
}

#[test]
fn test_code_gen_jne() {
    let src = "jne test\ntest:";
    let parsed = mk_parser().parse(src).into_output().unwrap();
    let assembled_code = gen_code(&parsed).unwrap();
    assert_eq!(assembled_code.bytes.len(), 9);
    assert_eq!(assembled_code.bytes[0], 0x74); // JNE opcode
}

#[test]
fn test_code_gen_jge() {
    let src = "jge end\nend:";
    let parsed = mk_parser().parse(src).into_output().unwrap();
    let assembled_code = gen_code(&parsed).unwrap();
    assert_eq!(assembled_code.bytes.len(), 9);
    assert_eq!(assembled_code.bytes[0], 0x75); // JGE opcode
}

#[test]
fn test_code_gen_jg() {
    let src = "jg start\nstart:";
    let parsed = mk_parser().parse(src).into_output().unwrap();
    let assembled_code = gen_code(&parsed).unwrap();
    assert_eq!(assembled_code.bytes.len(), 9);
    assert_eq!(assembled_code.bytes[0], 0x76); // JG opcode
}

#[test]
fn test_code_gen_cmov_variants() {
    let test_cases = [
        ("cmovle %rax, %rbx", 0x21),
        ("cmovl %rax, %rbx", 0x22),
        ("cmove %rax, %rbx", 0x23),
        ("cmovne %rax, %rbx", 0x24),
        ("cmovge %rax, %rbx", 0x25),
        ("cmovg %rax, %rbx", 0x26),
    ];

    for (src, expected_opcode) in test_cases {
        let parsed = mk_parser().parse(src).into_output().unwrap();
        let assembled_code = gen_code(&parsed).unwrap();
        assert_eq!(assembled_code.bytes.len(), 2);
        assert_eq!(assembled_code.bytes[0], expected_opcode);
        assert_eq!(assembled_code.bytes[1], 0x01); // rax=0, rbx=1
    }
}

#[test]
fn test_code_gen_mrmov_zero_displacement() {
    let src = "mrmovq (%rsp), %rax";
    let parsed = mk_parser().parse(src).into_output().unwrap();
    let assembled_code = gen_code(&parsed).unwrap();
    assert_eq!(assembled_code.bytes.len(), 10);
    assert_eq!(assembled_code.bytes[0], 0x50); // MRMOV opcode
    assert_eq!(assembled_code.bytes[1], 0x06); // rsp=6, rax=0
    // Displacement 0 in little-endian
    assert_eq!(assembled_code.bytes[2..10], [0, 0, 0, 0, 0, 0, 0, 0]);
}

#[test]
fn test_code_gen_rmmov_large_displacement() {
    let src = "rmmovq %r10, 256(%r11)";
    let parsed = mk_parser().parse(src).into_output().unwrap();
    let assembled_code = gen_code(&parsed).unwrap();
    assert_eq!(assembled_code.bytes.len(), 10);
    assert_eq!(assembled_code.bytes[0], 0x40); // RMMOV opcode
    assert_eq!(assembled_code.bytes[1], 0xAB); // r10=10, r11=11
    // Displacement 256 in little-endian
    assert_eq!(assembled_code.bytes[2..10], [0, 1, 0, 0, 0, 0, 0, 0]);
}

#[test]
fn test_code_gen_irmov_with_label() {
    let src = "irmovq target, %rax\ntarget:\nhalt";
    let parsed = mk_parser().parse(src).into_output().unwrap();
    let assembled_code = gen_code(&parsed).unwrap();
    assert_eq!(assembled_code.bytes.len(), 11);
    assert_eq!(assembled_code.bytes[0], 0x30); // IRMOV opcode
    assert_eq!(assembled_code.bytes[1], 0xF0); // rax=0, F for no src reg
    // Address 10 (target label location) in little-endian
    assert_eq!(assembled_code.bytes[2..10], [10, 0, 0, 0, 0, 0, 0, 0]);
}

#[test]
fn test_code_gen_push_all_registers() {
    let test_cases = [
        ("pushq %rax", 0x0F),
        ("pushq %rbx", 0x1F),
        ("pushq %rcx", 0x2F),
        ("pushq %rdx", 0x3F),
        ("pushq %rdi", 0x4F),
        ("pushq %rsi", 0x5F),
        ("pushq %rsp", 0x6F),
        ("pushq %rbp", 0x7F),
    ];

    for (src, expected_reg_byte) in test_cases {
        let parsed = mk_parser().parse(src).into_output().unwrap();
        let assembled_code = gen_code(&parsed).unwrap();
        assert_eq!(assembled_code.bytes.len(), 2);
        assert_eq!(assembled_code.bytes[0], 0xA0); // PUSH opcode
        assert_eq!(assembled_code.bytes[1], expected_reg_byte);
    }
}

#[test]
fn test_code_gen_pop_all_registers() {
    let test_cases = [
        ("popq %rax", 0x0F),
        ("popq %rbx", 0x1F),
        ("popq %rcx", 0x2F),
        ("popq %rdx", 0x3F),
        ("popq %rdi", 0x4F),
        ("popq %rsi", 0x5F),
        ("popq %rsp", 0x6F),
        ("popq %rbp", 0x7F),
    ];

    for (src, expected_reg_byte) in test_cases {
        let parsed = mk_parser().parse(src).into_output().unwrap();
        let assembled_code = gen_code(&parsed).unwrap();
        assert_eq!(assembled_code.bytes.len(), 2);
        assert_eq!(assembled_code.bytes[0], 0xB0); // POP opcode
        assert_eq!(assembled_code.bytes[1], expected_reg_byte);
    }
}

#[test]
fn test_code_gen_quad_negative() {
    let src = ".quad -42";
    let parsed = mk_parser().parse(src).into_output().unwrap();
    let assembled_code = gen_code(&parsed).unwrap();
    assert_eq!(assembled_code.bytes.len(), 8);
    // -42 in little-endian two's complement
    assert_eq!(
        assembled_code.bytes[0..8],
        [214, 255, 255, 255, 255, 255, 255, 255]
    );
}

#[test]
fn test_code_gen_quad_large_positive() {
    let src = ".quad 0x123456789ABCDEF0";
    let parsed = mk_parser().parse(src).into_output().unwrap();
    let assembled_code = gen_code(&parsed).unwrap();
    assert_eq!(assembled_code.bytes.len(), 8);
    // 0x123456789ABCDEF0 in little-endian
    assert_eq!(
        assembled_code.bytes[0..8],
        [0xF0, 0xDE, 0xBC, 0x9A, 0x78, 0x56, 0x34, 0x12]
    );
}

#[test]
fn test_code_gen_je() {
    let src = "je loop\nloop:";
    let parsed = mk_parser().parse(src).into_output().unwrap();
    let assembled_code = gen_code(&parsed).unwrap();
    assert_eq!(assembled_code.bytes.len(), 9);
    assert_eq!(assembled_code.bytes[0], 0x73); // JE opcode
}

#[test]
fn test_code_gen_call() {
    let src = "call func\nfunc:";
    let parsed = mk_parser().parse(src).into_output().unwrap();
    let assembled_code = gen_code(&parsed).unwrap();
    assert_eq!(assembled_code.bytes.len(), 9);
    assert_eq!(assembled_code.bytes[0], 0x80); // CALL opcode
}

#[test]
fn test_code_gen_ret() {
    let src = "ret";
    let parsed = mk_parser().parse(src).into_output().unwrap();
    let assembled_code = gen_code(&parsed).unwrap();
    assert_eq!(assembled_code.bytes.len(), 1);
    assert_eq!(assembled_code.bytes[0], 0x90); // RET opcode
}

#[test]
fn test_code_gen_push() {
    let src = "pushq %rbx";
    let parsed = mk_parser().parse(src).into_output().unwrap();
    let assembled_code = gen_code(&parsed).unwrap();
    assert_eq!(assembled_code.bytes.len(), 2);
    assert_eq!(assembled_code.bytes[0], 0xA0); // PUSH opcode
    assert_eq!(assembled_code.bytes[1], 0x1F); // rbx=1, F for no register
}

#[test]
fn test_code_gen_pop() {
    let src = "popq %rcx";
    let parsed = mk_parser().parse(src).into_output().unwrap();
    let assembled_code = gen_code(&parsed).unwrap();
    assert_eq!(assembled_code.bytes.len(), 2);
    assert_eq!(assembled_code.bytes[0], 0xB0); // POP opcode
    assert_eq!(assembled_code.bytes[1], 0x2F); // rcx=1, F for no register
}

#[test]
fn test_code_gen_align() {
    let src = ".align 8\nnop";
    let parsed = mk_parser().parse(src).into_output().unwrap();
    let assembled_code = gen_code(&parsed).unwrap();
    assert_eq!(assembled_code.bytes.len(), 1); // No padding needed
    assert_eq!(assembled_code.bytes[0], 0x10); // NOP opcode
}

#[test]
fn test_code_gen_multiple_instructions() {
    let src = "halt\nnop\nret";
    let parsed = mk_parser().parse(src).into_output().unwrap();
    let assembled_code = gen_code(&parsed).unwrap();
    assert_eq!(assembled_code.bytes.len(), 3);
    assert_eq!(assembled_code.bytes[0], 0x00); // HALT
    assert_eq!(assembled_code.bytes[1], 0x10); // NOP
    assert_eq!(assembled_code.bytes[2], 0x90); // RET
}

#[test]
fn test_code_gen_label_resolution() {
    let src = "irmovq start, %rax\nstart:\nhalt";
    let parsed = mk_parser().parse(src).into_output().unwrap();
    let assembled_code = gen_code(&parsed).unwrap();
    assert_eq!(assembled_code.bytes.len(), 11);
    // Address 10 (start label location) in little-endian
    assert_eq!(assembled_code.bytes[2..10], [10, 0, 0, 0, 0, 0, 0, 0]);
}

#[test]
fn test_code_gen_line_ranges() {
    let src = "halt\nnop\nret";
    let parsed = mk_parser().parse(src).into_output().unwrap();
    let assembled_code = gen_code(&parsed).unwrap();
    assert_eq!(assembled_code.line_ranges.len(), 3);
    assert_eq!(assembled_code.line_ranges[0], (0, 1)); // halt: bytes 0-1
    assert_eq!(assembled_code.line_ranges[1], (1, 2)); // nop: bytes 1-2
    assert_eq!(assembled_code.line_ranges[2], (2, 3)); // ret: bytes 2-3
}
#[test]
fn test_code_gen_complex_program() {
    let src = "
    irmovq $10, %rax
    irmovq $5, %rbx
    loop:
    subq %rbx, %rax
    jg loop
    halt
    ";
    let parsed = mk_parser().parse(src).into_output().unwrap();
    let assembled_code = gen_code(&parsed).unwrap();

    // Verify the structure
    assert_eq!(assembled_code.line_ranges.len(), 6); // 5 instructions

    // Check first instruction (irmovq $10, %rax)
    assert_eq!(assembled_code.bytes[0], 0x30); // IRMOV opcode
    assert_eq!(assembled_code.bytes[2..10], [10, 0, 0, 0, 0, 0, 0, 0]);

    // Check last instruction (halt)
    let halt_pos = assembled_code.line_ranges[5].0;
    assert_eq!(assembled_code.bytes[halt_pos], 0x00); // HALT opcode
}
