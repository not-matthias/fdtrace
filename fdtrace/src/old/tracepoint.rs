// https://github.com/aurae-runtime/aurae/blob/1d62cb044fc973b3b8006466c31ef3f35ecbca63/auraed/src/ebpf/kprobe/kprobe_program.rs

use anyhow::Context;
use aya::programs::{ProgramError, TracePoint};
use aya::Bpf;
use tracing::{trace, warn};

pub trait TracepointProgram<T: Clone + Send + 'static> {
    const PROGRAM_NAME: &'static str;
    const CATEGORY: &'static str;
    const EVENT: &'static str;
    const PERF_BUFFER: &'static str;

    fn load_and_attach(bpf: &mut Bpf) -> Result<(), anyhow::Error> {
        trace!("Loading eBPF program: {}", Self::PROGRAM_NAME);

        // Load the eBPF TracePoint program
        let program: &mut TracePoint = bpf
            .program_mut(Self::PROGRAM_NAME)
            .context("failed to get eBPF program")?
            .try_into()?;

        // Load the program
        match program.load() {
            Ok(_) => Ok(()),
            Err(ProgramError::AlreadyLoaded) => {
                warn!("Already loaded eBPF program {}", Self::PROGRAM_NAME);
                Ok(())
            }
            other => other,
        }?;

        // Attach to kernel trace event
        match program.attach(Self::CATEGORY, Self::EVENT) {
            Ok(_) => Ok(()),
            Err(ProgramError::AlreadyAttached) => {
                warn!("Already attached eBPF program {}", Self::PROGRAM_NAME);
                Ok(())
            }
            Err(e) => Err(e),
        }?;

        Ok(())
    }
}
