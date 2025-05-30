use log::{debug, error, info};
use y86_seq::assemble;
fn main() {
    env_logger::init();
    println!("Y86-64 Assembler");

    let src_file = std::env::args().nth(1).expect("No input file provided");
    let dest_file = match std::env::args().nth(2) {
        Some(file) => file,
        None => {
            let default_dest = format!("{}.o", src_file);
            println!("No output file provided, using default: {}", default_dest);
            default_dest
        }
    };

    println!("Reading source file: {}", src_file);
    let src_content = std::fs::read_to_string(&src_file).expect("Failed to read input file");
    debug!("Source content:\n{}", src_content);

    let assembly_result = assemble(&src_content, 0x1000);
    let Ok(output_bytes) = assembly_result else {
        error!("Assembly failed");
        std::process::exit(1);
    };

    // Write the output bytes to the destination file
    std::fs::write(&dest_file, &output_bytes).expect("Failed to write output file");
    info!("Output written to: {}", dest_file);
    println!("Assembly completed successfully.");
}
