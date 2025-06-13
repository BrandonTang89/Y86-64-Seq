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

    let (dissassembly, _log, _final_state) = simulate::<1024>(&mmap);

    println!("=========================");
    println!("Disassembly:");
    println!("=========================");
    for instruction in dissassembly {
        println!("{}", instruction);
    }
}
