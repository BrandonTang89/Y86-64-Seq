mod assemble;
pub(crate) mod ast;
mod parser;

use crate::assemble::{AssembledCode, gen_code};
use crate::parser::mk_parser;
use chumsky::prelude::*;

type ParseResult<'a> = Vec<ast::AssemblyLine<'a>>;
/// Invoke the parser and generate the assembled code from the provided assembly source code.
pub fn parse_and_gen(src_asm: &str) -> Result<(ParseResult, AssembledCode), String> {
    let parse_result = mk_parser()
        .parse(src_asm)
        .into_output()
        .expect("Failed to parse assembly code");

    let assembled_code = gen_code(&parse_result)?;
    Ok((parse_result, assembled_code))
}
