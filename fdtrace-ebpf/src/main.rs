#![no_std]
#![no_main]

use aya_ebpf::{
    helpers::{bpf_ktime_get_ns, bpf_probe_read_user_str_bytes},
    macros::{map, tracepoint},
    maps::{Array, PerfEventArray},
    programs::TracePointContext,
    EbpfContext,
};
#[allow(unused_imports)] use aya_log_ebpf::info;
use core::slice;
use fdtrace_common::SyscallLog;

mod ctx;
// mod open;

const BPF_PROG_SUCCESS: u32 = 0;
const BPF_PROG_FAILURE: u32 = 1;

// https://github.com/kov/lupa/blob/main/lupa-ebpf/src/main.rs#L13C40-L16C65
#[map]
static PID_TO_TRACE: Array<u64> = Array::with_max_entries(1, 0);

fn should_trace(pid: u64) -> bool {
    let to_track = match PID_TO_TRACE.get(0) {
        None => return false,
        Some(to_track) => *to_track,
    };

    pid == to_track
}

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

// sys/kernel/debug/tracing/events/<category>/<tracepoint>/format`

// [18:35] not-matthias:/home/not-matthias> sudo cat
// /sys/kernel/debug/tracing/events/raw_syscalls/sys_enter/format
// name: sys_enter
// ID: 392
// format:
// 	field:unsigned short common_type;	offset:0;	size:2;	signed:0;
// 	field:unsigned char common_flags;	offset:2;	size:1;	signed:0;
// 	field:unsigned char common_preempt_count;	offset:3;	size:1;	signed:0;
// 	field:int common_pid;	offset:4;	size:4;	signed:1;

// 	field:long id;	offset:8;	size:8;	signed:1;
// 	field:unsigned long args[6];	offset:16;	size:48;	signed:0;

// print fmt: "NR %ld (%lx, %lx, %lx, %lx, %lx, %lx)", REC->id, REC->args[0],
// REC->args[1], REC->args[2], REC->args[3], REC->args[4], REC->args[5]

// https://github.com/gmh5225/kunai/blob/b29af40fefb58e328ced9c367325d876497a4b7b/kunai-ebpf/src/probes/mmap.rs#L32\
//
// TODO: https://github.com/auseckas/bitblazr/blob/706b5e71131c701e0480c26c3495ef4fc9f6e6ac/probes/tracepoints-ebpf/src/tracepoints.rs#L162
unsafe fn try_fdtrace(ctx: TracePointContext) -> Result<u32, u32> {
    let args = slice::from_raw_parts(ctx.as_ptr() as *const usize, 2);

    // info!(&ctx, "args: {}", args[0]);

    let timestamp = bpf_ktime_get_ns();
    let syscall = args[1] as u32;
    let pid = ctx.pid();
    let cmd = ctx.command().map_err(|e| e as u32)?;

    // Get the arguments of 'close' (syscall id = 3)
    if syscall == 3 && false {
        // /sys/kernel/debug/tracing/events/syscalls/sys_enter_execve/format
        //
        // /sys/kernel/debug/tracing/events/syscalls/sys_enter_close/format
        // name: sys_enter_close
        // ID: 756
        // field:unsigned int fd;	offset:16;	size:8;	signed:0;

        let fd = ctx.read_at::<u64>(16).map_err(|e| e as u32)?;
        info!(&ctx, "fd: {}", fd);
    }

    // sudo cat /sys/kernel/debug/tracing/events/syscalls/sys_enter_open/format
    // sudo cat /sys/kernel/debug/tracing/events/

    // open
    if syscall == 2 {
        // let args = slice::from_raw_parts(ctx.as_ptr() as *const usize, 4);

        // https://github.com/enustah/fpbe/blob/25bbf1fd7c29f03a02b031ecad1b4a0411b0831c/fpbe-ebpf/src/openat.rs

        // field:int __syscall_nr;	offset:8;	size:4;	signed:1;
        // field:const char * filename;	offset:16;	size:8;	signed:0;
        // field:int flags;	offset:24;	size:8;	signed:0;
        // field:umode_t mode;	offset:32;	size:8;	signed:0;
        //
        // let name_ptr = ctx.read_at::<u64>(16).map_err(|e| e as u32)?;
        // let flags = ctx.read_at::<u64>(24).map_err(|e| e as u32)?;
        // let mode = ctx.read_at::<u64>(32).map_err(|e| e as u32)?;

        let file_name: u64 = ctx.read_at(16 + 16).map_err(|e| e as u32)?;
        info!(&ctx, "file_name: {}", file_name);

        // info!(
        //     &ctx,
        //     "arg0: {}, flags: {}, mode: {}, ptr: {:x}", args[1], args[1],
        // args[2], args[3] );

        // read the filename
        //

        // let value: u8 = aya_ebpf::helpers::bpf_probe_read_user(args[2] as
        // *const u8).map_err(|e| e as u32)?; info!(&ctx, "value: {}",
        // value);

        // let mut buf = [0u8; 64];
        // let result = aya_ebpf::helpers::bpf_probe_read_user_buf(args[2] as
        // *const u8, &mut buf).map_err(|e| e as u32)?; info!(&ctx,
        // "Read success");

        // set the last byte to 0
        // for val in &buf {
        //     info!(&ctx, "{}", *val);
        // }
        // let my_str = core::str::from_utf8(&buf).map_err(|_| 1_u32)?;

        // let mut buf = [0u8; 8];
        // let my_str_bytes =
        //     unsafe { aya_ebpf::helpers::bpf_probe_read_user_buf(args[2] as
        // *const _, &mut buf) }         .map_err(|e| e as u32)?;
        // let my_str = core::str::from_utf8(&buf).map_err(|_| 1_u32)?;
        // info!(&ctx, "filename: {}", my_str);

        // use aya_ebpf::{bpf_printk, helpers::bpf_get_current_comm};
        // let comm = bpf_get_current_comm().map_err(|e| e as u32)?;
        // bpf_printk!(
        //     b"arg: '%s' arg: '%s' arg: '%s' \n\0",
        //     args[1] as *const u8,
        //     args[2] as *const u8,
        //     args[3] as *const u8
        // );
    }

    if syscall == 0 && false
    /* read */
    {
        // field:int __syscall_nr;	offset:8;	size:4;	signed:1;
        // field:unsigned int fd;	offset:16;	size:8;	signed:0;
        // field:char * buf;	offset:24;	size:8;	signed:0;
        // field:size_t count;	offset:32;	size:8;	signed:0;

        let fd = ctx.read_at::<u64>(16).map_err(|e| e as u32)?;
        // let buf = ctx.read_at::<u64>(24).map_err(|e| e as u32)?;
        let count = ctx.read_at::<u64>(32).map_err(|e| e as u32)?;

        info!(&ctx, "fd: {} | count: {}", fd, count);
    }

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
