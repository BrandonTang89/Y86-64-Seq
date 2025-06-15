pub use crate::ast::*;
#[cfg(test)]
mod codegen_tests;

fn fill_immediate_little_endian(output: &mut [u8], value: i64) {
    let bytes = value.to_le_bytes();
    for (i, &byte) in bytes.iter().enumerate() {
        output[i] = byte;
    }
}

fn fill_imm_or_label(
    output: &mut [u8],
    value: LabOrImm<&str>,
    label_locations: &std::collections::HashMap<&str, i64>,
) -> Result<(), String> {
    match value {
        LabOrImm::Immediate(imm) => {
            fill_immediate_little_endian(output, imm);
            Ok(())
        }
        LabOrImm::Labelled(label) => {
            if let Some(&location) = label_locations.get(label) {
                fill_immediate_little_endian(output, location);
                Ok(())
            } else {
                Err(format!("Label '{}' not found", label))
            }
        }
    }
}

pub struct AssembledCode {
    /// The assembled code as a vector of bytes
    pub bytes: Vec<u8>,

    /// [start, end) byte locations for each instruction
    pub line_ranges: Vec<(usize, usize)>,
}

pub fn gen_code<'a>(ast: &Vec<BorrowedInstruction<'a>>) -> Result<AssembledCode, String> {
    let mut instruction_lengths: Vec<_> = ast
        .iter()
        .map(|line| {
            match line {
                Instruction::Label(_) => 0,        // Labels do not generate code
                Instruction::Directive(_, _) => 0, // Directives do not generate code
                Instruction::Halt => 1,
                Instruction::Nop => 1,
                Instruction::Irmov(_, _) => 10,
                Instruction::Rmmov(_, _, _) => 10,
                Instruction::Mrmov(_, _, _) => 10,
                Instruction::Binop(_, _, _) => 2,
                Instruction::Jmp(_, _) => 9,
                Instruction::Cmov(_, _, _) => 2,
                Instruction::Call(_) => 9,
                Instruction::Ret => 1,
                Instruction::Push(_) => 2,
                Instruction::Pop(_) => 2,
            }
        })
        .collect();

    let mut instruction_starts = vec![0; ast.len()];
    for i in 0..ast.len() {
        if i > 0 {
            instruction_starts[i] = instruction_starts[i - 1] + instruction_lengths[i - 1];
        }
        match ast[i] {
            Instruction::Directive(".align", align) => {
                let start = instruction_starts[i];
                let padding = ((-start) % align + align) % align;
                instruction_starts[i] = start;
                instruction_lengths[i] = padding;
            }
            Instruction::Directive(".quad", _) => {
                instruction_lengths[i] = 8;
            }
            _ => continue, // Other lines do not affect instruction starts
        }
    }

    let line_ranges = instruction_starts
        .iter()
        .zip(instruction_lengths.iter())
        .map(|(&start, &length)| (start as usize, (start + length) as usize))
        .collect::<Vec<_>>();

    let label_locations = ast
        .iter()
        .enumerate()
        .filter_map(|(i, line)| {
            if let &Instruction::Label(label) = line {
                Some((label, instruction_starts[i]))
            } else {
                None
            }
        })
        .collect::<std::collections::HashMap<_, _>>();

    let mut output_bytes: Vec<u8> = vec![
        0;
        (*instruction_starts.last().unwrap() + instruction_lengths.last().unwrap())
            as usize
    ];

    for (i, line) in ast.iter().enumerate() {
        let start = instruction_starts[i] as usize;
        match line {
            Instruction::Label(_) => continue, // Labels do not generate code
            Instruction::Directive(".quad", imm) => {
                fill_immediate_little_endian(&mut output_bytes[start..start + 8], *imm);
                continue; // .quad directive generates 8 bytes
            }
            Instruction::Directive(_, _) => continue, // Directives do not generate code

            Instruction::Halt => output_bytes[start] = 0x0 << 4, // HALT opcode
            Instruction::Nop => output_bytes[start] = 0x1 << 4,  // NOP opcode
            Instruction::Cmov(cond, src, dst) => {
                output_bytes[start] = 0x02 << 4 | (*cond as u8); // CMOV opcode
                output_bytes[start + 1] = (*src as u8) << 4 | (*dst as u8);
            }
            Instruction::Irmov(imm, reg) => {
                output_bytes[start] = 0x03 << 4; // IRMOVQ opcode
                output_bytes[start + 1] = 0xF0 | *reg as u8;
                fill_imm_or_label(
                    &mut output_bytes[start + 2..],
                    imm.clone(),
                    &label_locations,
                )?;
            }
            Instruction::Rmmov(src, offset, dst) => {
                output_bytes[start] = 0x04 << 4; // RMMOVQ opcode
                output_bytes[start + 1] = (*src as u8) << 4 | (*dst as u8);
                fill_immediate_little_endian(&mut output_bytes[start + 2..], *offset);
            }
            Instruction::Mrmov(offset, base, dst) => {
                output_bytes[start] = 0x05 << 4; // MRMOVQ opcode
                output_bytes[start + 1] = (*dst as u8) << 4 | (*base as u8);
                fill_immediate_little_endian(&mut output_bytes[start + 2..], *offset);
            }
            Instruction::Binop(op, src, dst) => {
                output_bytes[start] = 0x06 << 4 | (*op as u8); // BINOP opcode
                output_bytes[start + 1] = (*src as u8) << 4 | (*dst as u8);
            }
            Instruction::Jmp(cond, target) => {
                output_bytes[start] = 0x07 << 4 | (*cond as u8); // JMP opcode
                fill_imm_or_label(
                    &mut output_bytes[start + 1..],
                    target.clone(),
                    &label_locations,
                )?;
            }
            Instruction::Call(target) => {
                output_bytes[start] = 0x08 << 4; // CALL opcode
                fill_imm_or_label(
                    &mut output_bytes[start + 1..],
                    target.clone(),
                    &label_locations,
                )?;
            }
            Instruction::Ret => {
                output_bytes[start] = 0x09 << 4; // RET opcode
            }
            Instruction::Push(reg) => {
                output_bytes[start] = 0x0A << 4; // PUSH opcode
                output_bytes[start + 1] = (*reg as u8) << 4 | 0xF;
            }
            Instruction::Pop(reg) => {
                output_bytes[start] = 0x0B << 4; // POP opcode
                output_bytes[start + 1] = (*reg as u8) << 4 | 0xF;
            }
        }
    }

    Ok(AssembledCode {
        bytes: output_bytes,
        line_ranges,
    })
}
