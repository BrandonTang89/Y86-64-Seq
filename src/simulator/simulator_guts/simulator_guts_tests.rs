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

    // Helper function to create push instruction
    fn create_push_program(reg: Register) -> Vec<u8> {
        vec![
            0xa0,                    // opcode 0xa for push
            (reg as u8) << 4 | 0xf,  // register in upper nibble, lower nibble unused (0xf)
        ]
    }

    // Helper function to create pop instruction
    fn create_pop_program(reg: Register) -> Vec<u8> {
        vec![
            0xb0,                    // opcode 0xb for pop
            (reg as u8) << 4 | 0xf,  // register in upper nibble, lower nibble unused (0xf)
        ]
    }

    #[test]
    fn test_push_basic_operation() {
        let program = create_push_program(Register::Rax);
        let mut sim = Simulator::<1024>::new(&program);
        
        // Set up register value to push
        sim.registers[Register::Rax as usize] = 42;
        let initial_sp = sim.registers[Register::Rsp as usize];
        
        sim.run_single();
        
        // Check that value was written to memory at new SP location
        let new_sp = sim.registers[Register::Rsp as usize];
        assert_eq!(new_sp, initial_sp - 8, "SP should be decremented by 8");
        assert_eq!(sim.memory[new_sp as usize], 42, "Value should be stored at new SP location");
        
        // Check instruction pointer advancement
        assert_eq!(sim.instruction_pointer, 2);
        
        // Check that original register value is unchanged
        assert_eq!(sim.registers[Register::Rax as usize], 42);
    }

    #[test]
    fn test_pop_basic_operation() {
        let program = create_pop_program(Register::Rbx);
        let mut sim = Simulator::<1024>::new(&program);
        
        // Set up memory with a value to pop
        let initial_sp = sim.registers[Register::Rsp as usize];
        sim.memory[initial_sp as usize] = 99;
        
        sim.run_single();
        
        // Check that value was loaded into register
        assert_eq!(sim.registers[Register::Rbx as usize], 99, "Value should be loaded from stack");
        
        // Check that SP was incremented
        assert_eq!(sim.registers[Register::Rsp as usize], initial_sp + 8, "SP should be incremented by 8");
        
        // Check instruction pointer advancement
        assert_eq!(sim.instruction_pointer, 2);
    }

    #[test]
    fn test_push_rsp_edge_case() {
        let program = create_push_program(Register::Rsp);
        let mut sim = Simulator::<1024>::new(&program);
        
        let initial_sp = sim.registers[Register::Rsp as usize];
        
        sim.run_single();
        
        // The old SP value should be pushed onto the stack
        let new_sp = sim.registers[Register::Rsp as usize];
        assert_eq!(new_sp, initial_sp - 8, "SP should be decremented by 8");
        assert_eq!(sim.memory[new_sp as usize], initial_sp, "Old SP value should be pushed onto stack");
        
        // Check instruction pointer advancement
        assert_eq!(sim.instruction_pointer, 2);
    }

    #[test]
    fn test_pop_rsp_edge_case() {
        let program = create_pop_program(Register::Rsp);
        let mut sim = Simulator::<1024>::new(&program);
        
        // Set up memory with a value to pop into SP
        let initial_sp = sim.registers[Register::Rsp as usize];
        let target_sp_value = 500i64;
        sim.memory[initial_sp as usize] = target_sp_value;
        
        sim.run_single();
        
        // The value from memory should become the new SP (not SP + 8)
        assert_eq!(sim.registers[Register::Rsp as usize], target_sp_value, 
                   "SP should be set to the popped value, not SP+8");
        
        // Check instruction pointer advancement
        assert_eq!(sim.instruction_pointer, 2);
    }

    #[test]
    fn test_push_pop_round_trip() {
        // Create a program that pushes a value then pops it back
        let mut program = create_push_program(Register::Rax);
        program.extend(create_pop_program(Register::Rbx));
        
        let mut sim = Simulator::<1024>::new(&program);
        
        // Set up initial values
        sim.registers[Register::Rax as usize] = 123;
        sim.registers[Register::Rbx as usize] = 0;
        let initial_sp = sim.registers[Register::Rsp as usize];
        
        // Execute push
        sim.run_single();
        let after_push_sp = sim.registers[Register::Rsp as usize];
        assert_eq!(after_push_sp, initial_sp - 8);
        assert_eq!(sim.memory[after_push_sp as usize], 123);
        
        // Execute pop
        sim.run_single();
        
        // Check that value was correctly transferred
        assert_eq!(sim.registers[Register::Rbx as usize], 123, "Popped value should match pushed value");
        assert_eq!(sim.registers[Register::Rsp as usize], initial_sp, "SP should be back to original value");
        
        // Check instruction pointer
        assert_eq!(sim.instruction_pointer, 4);
    }

    #[test]
    fn test_push_pop_rsp_round_trip() {
        // Test the complex case: push %rsp, then pop %rsp
        let mut program = create_push_program(Register::Rsp);
        program.extend(create_pop_program(Register::Rsp));
        
        let mut sim = Simulator::<1024>::new(&program);
        
        let initial_sp = sim.registers[Register::Rsp as usize];
        
        // Execute push %rsp
        sim.run_single();
        let after_push_sp = sim.registers[Register::Rsp as usize];
        assert_eq!(after_push_sp, initial_sp - 8, "SP should be decremented after push");
        assert_eq!(sim.memory[after_push_sp as usize], initial_sp, "Old SP should be on stack");
        
        // Execute pop %rsp
        sim.run_single();
        
        // SP should now be restored to the original value
        assert_eq!(sim.registers[Register::Rsp as usize], initial_sp, 
                   "SP should be restored to original value after pop %rsp");
        
        // Check instruction pointer
        assert_eq!(sim.instruction_pointer, 4);
    }

    #[test]
    fn test_push_stack_bounds_error() {
        let program = create_push_program(Register::Rax);
        let mut sim = Simulator::<1024>::new(&program);
        
        // Set SP to a value that would cause underflow
        sim.registers[Register::Rsp as usize] = 0;
        sim.registers[Register::Rax as usize] = 42;
        
        sim.run_single();
        
        // Should result in an error state
        assert!(matches!(sim.state, Status::Error(_)), "Should be in error state due to stack underflow");
    }

    #[test]
    fn test_pop_stack_bounds_error() {
        let program = create_pop_program(Register::Rax);
        let mut sim = Simulator::<1024>::new(&program);
        
        // Set SP to a value that's out of bounds
        sim.registers[Register::Rsp as usize] = 1024; // Out of memory bounds
        
        sim.run_single();
        
        // Should result in an error state
        assert!(matches!(sim.state, Status::Error(_)), "Should be in error state due to invalid SP");
    }

    #[test]
    fn test_push_logs_changes_correctly() {
        let program = create_push_program(Register::Rcx);
        let mut sim = Simulator::<1024>::new(&program);
        
        sim.registers[Register::Rcx as usize] = 777;
        let initial_sp = sim.registers[Register::Rsp as usize];
        
        sim.run_single();
        
        // Should have 3 log entries: memory change, SP change, IP change
        assert_eq!(sim.log.len(), 3);
        
        // Check memory change log
        if let (0, AtomicChange::Memory { addr, value }) = &sim.log[0] {
            assert_eq!(*addr, initial_sp - 8);
            assert_eq!(*value, 777);
        } else {
            panic!("Expected memory change as first log entry");
        }
        
        // Check SP change log
        if let (0, AtomicChange::Register { reg, value }) = &sim.log[1] {
            assert_eq!(*reg, Register::Rsp);
            assert_eq!(*value, initial_sp - 8);
        } else {
            panic!("Expected SP register change as second log entry");
        }
        
        // Check IP change log
        if let (0, AtomicChange::InstructionPointer { ip }) = &sim.log[2] {
            assert_eq!(*ip, 2);
        } else {
            panic!("Expected IP change as third log entry");
        }
    }

    #[test]
    fn test_pop_logs_changes_correctly() {
        let program = create_pop_program(Register::Rdx);
        let mut sim = Simulator::<1024>::new(&program);
        
        let initial_sp = sim.registers[Register::Rsp as usize];
        sim.memory[initial_sp as usize] = 888;
        
        sim.run_single();
        
        // Should have 3 log entries: register change, SP change, IP change
        assert_eq!(sim.log.len(), 3);
        
        // Check target register change log
        if let (0, AtomicChange::Register { reg, value }) = &sim.log[0] {
            assert_eq!(*reg, Register::Rdx);
            assert_eq!(*value, 888);
        } else {
            panic!("Expected target register change as first log entry");
        }
        
        // Check SP change log
        if let (0, AtomicChange::Register { reg, value }) = &sim.log[1] {
            assert_eq!(*reg, Register::Rsp);
            assert_eq!(*value, initial_sp + 8);
        } else {
            panic!("Expected SP register change as second log entry");
        }
        
        // Check IP change log
        if let (0, AtomicChange::InstructionPointer { ip }) = &sim.log[2] {
            assert_eq!(*ip, 2);
        } else {
            panic!("Expected IP change as third log entry");
        }
    }

    #[test]
    fn test_pop_rsp_logs_changes_correctly() {
        let program = create_pop_program(Register::Rsp);
        let mut sim = Simulator::<1024>::new(&program);
        
        let initial_sp = sim.registers[Register::Rsp as usize];
        let target_value = 300i64;
        sim.memory[initial_sp as usize] = target_value;
        
        sim.run_single();
        
        // Should have 2 log entries: SP change, IP change (no separate SP+8 change)
        assert_eq!(sim.log.len(), 2);
        
        // Check SP change log (should be set to popped value)
        if let (0, AtomicChange::Register { reg, value }) = &sim.log[0] {
            assert_eq!(*reg, Register::Rsp);
            assert_eq!(*value, target_value);
        } else {
            panic!("Expected SP register change as first log entry");
        }
        
        // Check IP change log
        if let (0, AtomicChange::InstructionPointer { ip }) = &sim.log[1] {
            assert_eq!(*ip, 2);
        } else {
            panic!("Expected IP change as second log entry");
        }
    }
}
