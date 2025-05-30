pub(crate) mod ast;
mod parser;
use crate::parser::mk_parser;
use chumsky::prelude::*;

pub fn assemble(src: &str, highest_addr: usize) -> Result<Vec<u8>, String> {
    let memory: Vec<u8> = Vec::with_capacity(highest_addr);

    let parse_result = mk_parser()
        .parse(src)
        .into_output()
        .expect("Failed to parse assembly code");

    for line in parse_result {
        eprintln!("LINE: {:?}", line);
    }

    Ok(memory)
}
