use memmap2::Mmap;
use y86_seq::simulate;

fn main() {
    colour::println_bold!("Y86-64 Instruction Level Simulator");
    let src_file = std::env::args().nth(1).expect("No input file provided");

    let file = std::fs::File::open(&src_file)
        .unwrap_or_else(|_| panic!("Failed to open input file: {}", src_file));

    let mmap = unsafe {
        Mmap::map(&file).unwrap_or_else(|_| panic!("Failed to memory-map the file: {}", src_file))
    };

    let (dissassembly, log, _final_state) = simulate::<1024>(&mmap);
    let diassembly_width = dissassembly
        .iter()
        .map(|(_, line)| format!("{}", line).len())
        .max()
        .unwrap_or(0)
        + 2; // Add padding

    println!("=========================");
    println!("Simulation:");
    println!("=========================");
    for ((addr, instruction), changes) in dissassembly.into_iter().zip(log) {
        println!(
            "{} {:diassembly_width$} | {}",
            format!("{:04x}", addr),
            format!("{}", instruction),
            changes
                .first()
                .map(|change| change.to_string())
                .unwrap_or_else(Default::default)
        );
        for change in changes.iter().skip(1) {
            println!(
                "{:04} {:diassembly_width$} | {}",
                "",
                "",
                change.to_string()
            );
        }
    }
}
