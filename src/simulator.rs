pub mod simulator_guts;
use simulator_guts::Simulator;

type SimulationResult<'a, const MEM_SIZE: usize> = Simulator<'a, MEM_SIZE>;
/// Run Simulator Until Halt or Error
pub fn simulate<'a, const MEM_SIZE: usize>(src: &'a [u8]) -> SimulationResult<'a, MEM_SIZE> {
    let mut state = Simulator::<'a, MEM_SIZE>::new(src);
    loop {
        state.run_single();
        let (ip, asm_line) = state.disassembly.last().unwrap();
        println!("Last Line: {}: {}", ip, asm_line);
        
        if state.state == simulator_guts::Status::Halted {
            return state;
        }
        
        if let simulator_guts::Status::Error(e) = &state.state {
            println!("Error: {}", e);
            return state;
        }
        
    }
}