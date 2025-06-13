use colour::{println_bold, red_ln};
use y86_seq::parse_and_gen;
fn main() {
    println_bold!("Y86-64 Assembler");
    let src_file = std::env::args().nth(1).expect("No input file provided");
    let dest_file = match std::env::args().nth(2) {
        Some(file) => file,
        None => {
            let default_dest = format!("{}o", src_file);
            default_dest
        }
    };
    println!("Input file: {}", src_file);
    let src_content = std::fs::read_to_string(&src_file).expect("Failed to read input file");

    println!("=========================");
    println!("Assembly Code:");
    println!("=========================");
    println!("{}", src_content);

    let res = parse_and_gen(&src_content);
    if let Err(e) = res {
        red_ln!("{}", e);
        std::process::exit(1);
    }
    let (parse_result, assembly_result) = res.unwrap();

    println!("=========================");
    println!("Generated Code:");
    println!("=========================");

    let line_width = parse_result
        .iter()
        .map(|line| format!("{}", line).len())
        .max()
        .unwrap_or(0)
        + 2; // Add padding

    for (&line, &(start, end)) in parse_result.iter().zip(assembly_result.line_ranges.iter()) {
        println!(
            "{:line_width$} | {}",
            format!("{}", line),
            assembly_result.bytes[start..end]
                .iter()
                .map(|b| format!("{:02x}", b))
                .collect::<Vec<_>>()
                .join(" ")
        );
    }
    println!();

    let output_bytes = assembly_result.bytes;
    println!("Writing output to: {}", dest_file);

    std::fs::write(&dest_file, &output_bytes).expect("Failed to write output file");
}
