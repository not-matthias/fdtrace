#![no_std]
#![no_main]

use aya_ebpf::{
    helpers::bpf_ktime_get_ns,
    macros::{map, tracepoint},
    maps::PerfEventArray,
    programs::TracePointContext,
    EbpfContext,
};
#[allow(unused_imports)] use aya_log_ebpf::info;
use core::slice;
use fdtrace_common::SyscallLog;

const BPF_PROG_SUCCESS: u32 = 0;
const BPF_PROG_FAILURE: u32 = 1;

// For kernel > 5.8 use RingBuf.
// TODO: impl this
//
// For kernel > 4.3 use PerfEventArray.
#[map(name = "EVENTS")]
static EVENTS: PerfEventArray<SyscallLog> = PerfEventArray::<SyscallLog>::new(0);

#[tracepoint]
pub fn fdtrace(ctx: TracePointContext) -> u32 {
    match unsafe { try_fdtrace(ctx) } {
        Ok(ret) => ret,
        Err(_) => BPF_PROG_FAILURE,
    }
}

// https://github.com/gmh5225/kunai/blob/b29af40fefb58e328ced9c367325d876497a4b7b/kunai-ebpf/src/probes/mmap.rs#L32
unsafe fn try_fdtrace(ctx: TracePointContext) -> Result<u32, u32> {
    // let args = slice::from_raw_parts(ctx.as_ptr() as *const usize, 2);
    // info!(&ctx, "args: {}", args[0]);

    let timestamp = bpf_ktime_get_ns();
    let syscall = args[1] as u32;
    let pid = ctx.pid();
    let cmd = ctx.command().map_err(|e| e as u32)?;

    let log = SyscallLog {
        pid,
        id: syscall,
        ts: timestamp,
        args: (),
        cmd,
    };

    // info!(
    //     &ctx,
    //     "ts: {} | id: {} | pid: {} | cmd: {}",
    //     timestamp,
    //     syscall,
    //     pid,
    //     log.cmd()
    // );

    EVENTS.output(&ctx, &log, 0);

    Ok(BPF_PROG_SUCCESS)
}

#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
