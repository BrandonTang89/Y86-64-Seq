#[test]
fn integration_tests_simple() {
    use y86_seq::assembler::parse_and_gen;
    use y86_seq::simulator::simulate;

    let src_asm = r#"
nop
nop
rrmovq %rax, %rbx
halt
        "#;

    let parse_result = parse_and_gen(src_asm);
    assert!(
        parse_result.is_ok(),
        "Parsing failed: {:?}",
        parse_result.err()
    );
    let machine_code = parse_result.unwrap().1.bytes;
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
        simulator.instruction_pointer, 3,
        "Instruction pointer should be at 3 after halting on 4th instruction"
    );
}
