// src/simulator/simulator_guts/simulator_guts_tests.rs
#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::ast::{BinaryOp, Register};

    fn create_binop_program(op: BinaryOp, src: Register, dest: Register) -> Vec<u8> {
        let func_code = match op {
            BinaryOp::Add => 0x0,
            BinaryOp::Sub => 0x1,
            BinaryOp::And => 0x2,
            BinaryOp::Xor => 0x3,
        };

        vec![
            0x60 | func_code,                // opcode 0x6 with function code
            (src as u8) << 4 | (dest as u8), // source and destination registers
        ]
    }

    #[test]
    fn test_binop_add_positive_numbers() {
        let program = create_binop_program(BinaryOp::Add, Register::Rax, Register::Rbx);
        let mut sim = Simulator::<1024>::new(&program);

        // Set up initial register values
        sim.registers[Register::Rax as usize] = 5;
        sim.registers[Register::Rbx as usize] = 3;

        sim.run_single();

        // Check result
        assert_eq!(sim.registers[Register::Rbx as usize], 8);

        // Check condition codes - should have no flags set
        assert_eq!(sim.condition_code & CARRY_MASK, 0); // No carry
        assert_eq!(sim.condition_code & ZERO_MASK, 0); // Not zero
        assert_eq!(sim.condition_code & SIGN_MASK, 0); // Positive
        assert_eq!(sim.condition_code & OVERFLOW_MASK, 0); // No overflow

        // Check instruction pointer advancement
        assert_eq!(sim.instruction_pointer, 2);
    }

    #[test]
    fn test_binop_add_with_carry() {
        let program = create_binop_program(BinaryOp::Add, Register::Rax, Register::Rbx);
        let mut sim = Simulator::<1024>::new(&program);

        // Set up values that will cause carry
        sim.registers[Register::Rax as usize] = i64::MAX;
        sim.registers[Register::Rbx as usize] = 1;

        sim.run_single();

        // Check result (should wrap around)
        assert_eq!(sim.registers[Register::Rbx as usize], i64::MIN);

        // Check condition codes
        assert_ne!(sim.condition_code & CARRY_MASK, 0); // Should have carry
        assert_eq!(sim.condition_code & ZERO_MASK, 0); // Not zero
        assert_ne!(sim.condition_code & SIGN_MASK, 0); // Negative result
        assert_ne!(sim.condition_code & OVERFLOW_MASK, 0); // Should have overflow
    }

    #[test]
    fn test_binop_add_zero_result() {
        let program = create_binop_program(BinaryOp::Add, Register::Rax, Register::Rbx);
        let mut sim = Simulator::<1024>::new(&program);

        sim.registers[Register::Rax as usize] = -5;
        sim.registers[Register::Rbx as usize] = 5;

        sim.run_single();

        // Check result
        assert_eq!(sim.registers[Register::Rbx as usize], 0);

        // Check condition codes
        assert_eq!(sim.condition_code & CARRY_MASK, 0); // No carry
        assert_ne!(sim.condition_code & ZERO_MASK, 0); // Should be zero
        assert_eq!(sim.condition_code & SIGN_MASK, 0); // Not negative
        assert_eq!(sim.condition_code & OVERFLOW_MASK, 0); // No overflow
    }

    #[test]
    fn test_binop_sub_basic() {
        let program = create_binop_program(BinaryOp::Sub, Register::Rax, Register::Rbx);
        let mut sim = Simulator::<1024>::new(&program);

        sim.registers[Register::Rax as usize] = 3; // This gets negated
        sim.registers[Register::Rbx as usize] = 10;

        sim.run_single();

        // Result should be -3 + 10 = 7
        assert_eq!(sim.registers[Register::Rbx as usize], 7);

        // Check condition codes
        assert_eq!(sim.condition_code & CARRY_MASK, 0); // No carry
        assert_eq!(sim.condition_code & ZERO_MASK, 0); // Not zero
        assert_eq!(sim.condition_code & SIGN_MASK, 0); // Positive
        assert_eq!(sim.condition_code & OVERFLOW_MASK, 0); // No overflow
    }

    #[test]
    fn test_binop_and_operation() {
        let program = create_binop_program(BinaryOp::And, Register::Rax, Register::Rbx);
        let mut sim = Simulator::<1024>::new(&program);

        // Set up some condition codes first to verify they're preserved
        sim.condition_code = CARRY_MASK | OVERFLOW_MASK;

        sim.registers[Register::Rax as usize] = 0b1100;
        sim.registers[Register::Rbx as usize] = 0b1010;

        sim.run_single();

        // Result should be 0b1000 = 8
        assert_eq!(sim.registers[Register::Rbx as usize], 8);

        // Check that carry and overflow flags are preserved
        assert_ne!(sim.condition_code & CARRY_MASK, 0);
        assert_ne!(sim.condition_code & OVERFLOW_MASK, 0);

        // Check other flags are set correctly
        assert_eq!(sim.condition_code & ZERO_MASK, 0); // Not zero
        assert_eq!(sim.condition_code & SIGN_MASK, 0); // Positive
    }

    #[test]
    fn test_binop_xor_operation() {
        let program = create_binop_program(BinaryOp::Xor, Register::Rax, Register::Rbx);
        let mut sim = Simulator::<1024>::new(&program);

        // Set up some condition codes to verify they're preserved
        sim.condition_code = CARRY_MASK | OVERFLOW_MASK;

        sim.registers[Register::Rax as usize] = 0b1100;
        sim.registers[Register::Rbx as usize] = 0b1010;

        sim.run_single();

        // Result should be 0b0110 = 6
        assert_eq!(sim.registers[Register::Rbx as usize], 6);

        // Check that carry and overflow flags are preserved for XOR
        assert_ne!(sim.condition_code & CARRY_MASK, 0);
        assert_ne!(sim.condition_code & OVERFLOW_MASK, 0);
        assert_eq!(sim.condition_code & ZERO_MASK, 0); // Not zero
        assert_eq!(sim.condition_code & SIGN_MASK, 0); // Positive
    }

    #[test]
    fn test_binop_xor_same_values() {
        let program = create_binop_program(BinaryOp::Xor, Register::Rax, Register::Rbx);
        let mut sim = Simulator::<1024>::new(&program);

        sim.registers[Register::Rax as usize] = 42;
        sim.registers[Register::Rbx as usize] = 42;

        sim.run_single();

        // XOR of same values should be 0
        assert_eq!(sim.registers[Register::Rbx as usize], 0);

        // Should set zero flag
        assert_ne!(sim.condition_code & ZERO_MASK, 0);
        assert_eq!(sim.condition_code & SIGN_MASK, 0); // Not negative
        assert_eq!(sim.condition_code & CARRY_MASK, 0); // Carry unchanged
        assert_eq!(sim.condition_code & OVERFLOW_MASK, 0); // Overflow unchanged
    }

    #[test]
    fn test_binop_negative_result() {
        let program = create_binop_program(BinaryOp::Add, Register::Rax, Register::Rbx);
        let mut sim = Simulator::<1024>::new(&program);

        sim.registers[Register::Rax as usize] = -10;
        sim.registers[Register::Rbx as usize] = -5;

        sim.run_single();

        // Result should be -15
        assert_eq!(sim.registers[Register::Rbx as usize], -15);

        // Check sign flag is set
        assert_ne!(sim.condition_code & SIGN_MASK, 0);
        assert_eq!(sim.condition_code & ZERO_MASK, 0); // Not zero
        assert_eq!(sim.condition_code & CARRY_MASK, 0); // No carry
        assert_eq!(sim.condition_code & OVERFLOW_MASK, 0); // No overflow
    }

    #[test]
    fn test_binop_logs_changes_correctly() {
        let program = create_binop_program(BinaryOp::Add, Register::Rax, Register::Rbx);
        let mut sim = Simulator::<1024>::new(&program);

        sim.registers[Register::Rax as usize] = 5;
        sim.registers[Register::Rbx as usize] = 3;

        sim.run_single();

        // Should have 3 log entries: register change, condition code change, IP change
        assert_eq!(sim.log.len(), 3);

        // Check register change log
        if let (0, AtomicChange::Register { reg, value }) = &sim.log[0] {
            assert_eq!(*reg, Register::Rbx);
            assert_eq!(*value, 8);
        } else {
            panic!("Expected register change as first log entry");
        }

        // Check condition code change log
        if let (0, AtomicChange::ConditionCode { cc: _ }) = &sim.log[1] {
            // Just verify it's a condition code change
        } else {
            panic!("Expected condition code change as second log entry");
        }

        // Check instruction pointer change log
        if let (0, AtomicChange::InstructionPointer { ip }) = &sim.log[2] {
            assert_eq!(*ip, 2);
        } else {
            panic!("Expected IP change as third log entry");
        }
    }
}
