use aya::programs::KProbe;
#[rustfmt::skip]
use log::{debug, warn};
use aya::{
    include_bytes_aligned,
    programs::{Xdp, XdpFlags},
    Bpf,
};
use aya_log::BpfLogger;
use tokio::signal;

// https://github.com/aurae-runtime/aurae/blob/1d62cb044fc973b3b8006466c31ef3f35ecbca63/auraed/src/ebpf/tracepoint/tracepoint_program.rs

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    // Bump the memlock rlimit. This is needed for older kernels that don't
    // use the new memcg based accounting, see https://lwn.net/Articles/837122/
    let rlim = libc::rlimit {
        rlim_cur: libc::RLIM_INFINITY,
        rlim_max: libc::RLIM_INFINITY,
    };
    let ret = unsafe { libc::setrlimit(libc::RLIMIT_MEMLOCK, &rlim) };
    if ret != 0 {
        debug!("remove limit on locked memory failed, ret is: {}", ret);
    }

    // Include the compiled BPF program in the binary.
    //
    let mut ebpf = Bpf::load(include_bytes_aligned!(
        "../../target/bpfel-unknown-none/release/fdtrace"
    ))?;
    BpfLogger::init(&mut ebpf)?;

    // Note: this name has to be the same as in the ebpf program.
    let program = ebpf.program_mut("xdp_hello").unwrap();
    log::info!("Got prgoram");

    let program: &mut Xdp = program.try_into()?;
    program.load()?;
    program.attach("wlp166s0", XdpFlags::default())?;

    let ctrl_c = signal::ctrl_c();
    println!("Waiting for Ctrl-C...");
    ctrl_c.await?;
    println!("Exiting...");

    Ok(())
}
