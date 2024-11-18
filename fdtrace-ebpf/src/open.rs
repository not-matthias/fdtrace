#[tracepoint]
pub fn fdtrace(ctx: TracePointContext) -> u32 {
    match unsafe { try_fdtrace(ctx) } {
        Ok(ret) => ret,
        Err(_) => BPF_PROG_FAILURE,
    }
}
