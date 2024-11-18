// https://github.com/aurae-runtime/aurae/blob/1d62cb044fc973b3b8006466c31ef3f35ecbca63/auraed/src/ebpf/kprobe/kprobe_program.rs

pub trait KProbeProgram<T: Clone + Send + 'static> {
    const PROGRAM_NAME: &'static str;
    const FUNCTION_NAME: &'static str;
    const PERF_BUFFER: &'static str;

    fn load_and_attach(bpf: &mut Bpf) -> Result<(), anyhow::Error> {
        trace!("Loading eBPF program: {}", Self::PROGRAM_NAME);

        // Load the eBPF TracePoint program
        let program: &mut KProbe = bpf
            .program_mut(Self::PROGRAM_NAME)
            .ok_or_else(|| anyhow::anyhow!("failed to get eBPF program"))?
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
        match program.attach(Self::FUNCTION_NAME, 0) {
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
