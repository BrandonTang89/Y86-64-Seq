use super::{AtomicChange, Status};
use core::fmt;

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Status::Running => write!(f, "Running"),
            Status::Halted => write!(f, "Halted"),
            Status::Error(msg) => write!(f, "Error: {}", msg),
        }
    }
}

impl fmt::Display for AtomicChange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AtomicChange::Register { reg, value } => write!(f, "{} = {}", reg, value),
            AtomicChange::Memory { addr, value } => write!(f, "Mem({}) = {}", addr, value),
            AtomicChange::InstructionPointer { ip } => write!(f, "RIP = {}", ip),
            AtomicChange::ConditionCode { cc } => write!(f, "CC = {:04b}", cc),
            AtomicChange::State { status } => write!(f, "Status = {}", status),
        }
    }
}
