pub(crate) mod ast;
mod parser;

use crate::{
    ast::{AssemblyLine, LabOrImm},
    parser::mk_parser,
};
use chumsky::prelude::*;

fn fill_immediate_little_endian(output: &mut [u8], value: i64) {
    let bytes = value.to_le_bytes();
    for (i, &byte) in bytes.iter().enumerate() {
        output[i] = byte;
    }
}

fn code_gen<'a>(ast: &Vec<AssemblyLine<'a>>) -> Result<Vec<u8>, String> {
    let mut instruction_lengths: Vec<_> = ast
        .iter()
        .map(|line| {
            match line {
                AssemblyLine::Label(_) => 0,        // Labels do not generate code
                AssemblyLine::Directive(_, _) => 0, // Directives do not generate code
                AssemblyLine::Halt => 1,
                AssemblyLine::Nop => 1,
                AssemblyLine::Rrmov(_, _) => 2,
                AssemblyLine::Irmov(_, _) => 10,
                AssemblyLine::Rmmov(_, _, _) => 10,
                AssemblyLine::Mrmov(_, _, _) => 10,
                AssemblyLine::Binop(_, _, _) => 2,
                AssemblyLine::Jmp(_, _) => 9,
                AssemblyLine::Cmov(_, _, _) => 2,
                AssemblyLine::Call(_) => 9,
                AssemblyLine::Ret => 1,
                AssemblyLine::Push(_) => 2,
                AssemblyLine::Pop(_) => 2,
            }
        })
        .collect();

    let mut instruction_starts = vec![0; ast.len()];
    for i in 1..ast.len() {
        match ast[i - 1] {
            AssemblyLine::Directive(".align", align) => {
                let end = instruction_starts[i - 1] + instruction_lengths[i - 1];
                let padding = align - (end % align);
                instruction_starts[i] = end + padding;
                instruction_lengths[i] = padding;
            }
            AssemblyLine::Directive(".quad", _) => {
                instruction_starts[i] = instruction_starts[i - 1] + 8;
                instruction_lengths[i] = 8;
            }
            _ => {
                instruction_starts[i] = instruction_starts[i - 1] + instruction_lengths[i - 1];
            }
        }
    }

    let label_locations = ast
        .iter()
        .enumerate()
        .filter_map(|(i, line)| {
            if let AssemblyLine::Label(label) = line {
                Some((label, instruction_starts[i]))
            } else {
                None
            }
        })
        .collect::<std::collections::HashMap<_, _>>();

    let mut output: Vec<u8> = vec![
        0;
        (*instruction_starts.last().unwrap() + instruction_lengths.last().unwrap())
            as usize
    ];

    for (i, line) in ast.iter().enumerate() {
        let start = instruction_starts[i] as usize;
        match line {
            AssemblyLine::Halt => output[start] = 0x0 << 4, // HALT opcode
            AssemblyLine::Nop => output[start] = 0x1 << 4,  // NOP opcode
            AssemblyLine::Rrmov(src, dst) => {
                output[start] = 0x2 << 4; // RRMOVQ opcode
                output[start + 1] = (*src as u8) << 4 | (*dst as u8);
            }
            AssemblyLine::Irmov(imm, reg) => {
                output[start] = 0x03 << 4; // IRMOVQ opcode
                output[start + 1] = *reg as u8;

                match imm {
                    LabOrImm::Immediate(value) => {
                        fill_immediate_little_endian(&mut output[start + 2..], *value);
                    }
                    LabOrImm::Labelled(label) => {
                        if let Some(&location) = label_locations.get(label) {
                            fill_immediate_little_endian(&mut output[start + 2..], location as i64);
                        } else {
                            eprintln!("Error: Label '{}' not found", label);
                            return Err(format!("Label '{}' not found", label));
                        }
                    }
                }
            }
            AssemblyLine::Rmmov(src, offset, base) => {
                output[start] = 0x04; // RMMOVQ opcode
                output[start + 1] = (*src as u8) << 4 | (*base as u8);
                fill_immediate_little_endian(&mut output[start + 2..], *offset);
            }

            _ => {
                todo!("Implement other assembly line types");
                eprintln!("Unhandled assembly line: {:?}", line);
            }
        }
    }

    Ok(output)
}

pub fn assemble(src_asm: &str) -> Result<Vec<u8>, String> {
    let parse_result = mk_parser()
        .parse(src_asm)
        .into_output()
        .expect("Failed to parse assembly code");

    for line in parse_result.iter() {
        eprintln!("LINE: {:?}", line);
    }

    let output_bytes = code_gen(&parse_result)?;
    Ok(output_bytes)
}
