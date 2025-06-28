use itertools::Itertools;
use memmap2::Mmap;
use y86_seq::simulator::simulate;

/// Memory-maps an input file and simulates the Y86-64 instructions contained within it.
fn main() {
    colour::println_bold!("Y86-64 Instruction Level Simulator");
    let src_file = std::env::args().nth(1).expect("No input file provided");

    let file = std::fs::File::open(&src_file)
        .unwrap_or_else(|_| panic!("Failed to open input file: {}", src_file));

    let mmap = unsafe {
        Mmap::map(&file).unwrap_or_else(|_| panic!("Failed to memory-map the file: {}", src_file))
    };

    let final_state = simulate::<1024>(&mmap);
    let diassembly_width = final_state
        .disassembly
        .iter()
        .map(|(_, line)| format!("{}", line).len())
        .max()
        .unwrap_or(0)
        + 2; // Add padding

    println!("=========================");
    println!("Simulation:");
    println!("=========================");
    let disassembly = final_state.disassembly.iter();
    let mut log = final_state.log.iter();
    for (id, (addr, instruction)) in disassembly.enumerate() {
        let changes = log
            .take_while_ref(|(log_id, _)| *log_id == id)
            .map(|(_, change)| change)
            .collect::<Vec<_>>();

        println!(
            "{:04x} {:diassembly_width$} | {}",
            addr,
            format!("{}", instruction),
            changes
                .first()
                .map(|change| change.to_string())
                .unwrap_or_else(Default::default)
        );

        for change in changes.iter().skip(1) {
            println!("{:04} {:diassembly_width$} | {}", "", "", change);
        }
    }
}
