mod assemble;
pub(crate) mod ast;
mod parser;

use crate::assemble::{AssembledCode, gen_code};
use crate::parser::mk_parser;
use chumsky::prelude::*;

type ParseResult<'a> = Vec<ast::AssemblyLine<'a>>;

/// Handles errors from the parser and formats them for display.
fn handle_error<'a>(src_asm: &str, errors: Vec<chumsky::error::Simple<'a, char>>) -> String {
    let span = errors.first().unwrap().span();
    let range = span.into_range();
    let found = errors.first().unwrap().found();

    let lines = src_asm.lines().collect::<Vec<_>>();
    let line_prefix_len = lines // the offset of each line in the source code
        .iter()
        .map(|line| line.len())
        .scan(0, |acc, len| {
            let old = *acc;
            *acc += len + 1; // +1 for the newline character
            Some(old)
        })
        .collect::<Vec<_>>();

    let relevant_lines = lines // lines with the error
        .into_iter()
        .zip(line_prefix_len.into_iter())
        .zip(1..)
        .map(|((line, prefix_len), line_number)| (line, prefix_len, line_number))
        .skip_while(|(line, prefix_len, _)| prefix_len + line.len() - 1 < range.start)
        .take_while(|(_, prefix_len, _)| prefix_len < &range.end)
        .map(|(line, _, line_number)| format!("Line {}: {}", line_number, line))
        .collect::<Vec<_>>();

    format!(
        "Parsing Error:\n{}\nFound: {}",
        relevant_lines.join("\n"),
        found.map(|c| c.to_string()).unwrap_or("EOF".to_string())
    )
}

/// Invoke the parser and generate the assembled code from the provided assembly source code.
pub fn parse_and_gen(src_asm: &str) -> Result<(ParseResult, AssembledCode), String> {
    let parse_result = mk_parser().parse(src_asm);

    if parse_result.has_output() {
        let ast = parse_result.into_output().unwrap();
        let assembled_code = gen_code(&ast)?;
        Ok((ast, assembled_code))
    } else {
        let errors = parse_result.into_errors();
        Err(handle_error(src_asm, errors))
    }
}
