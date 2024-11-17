use aya::{
    include_bytes_aligned, maps::AsyncPerfEventArray, programs::TracePoint, util::online_cpus, Ebpf,
};
use aya_log::EbpfLogger;
use bytes::BytesMut;
use fdtrace_common::SyscallLog;
use tokio::{signal, task};

mod args;

// https://github.com/aurae-runtime/aurae/blob/1d62cb044fc973b3b8006466c31ef3f35ecbca63/auraed/src/ebpf/tracepoint/tracepoint_program.rs

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    // let target = Box::new(std::fs::File::create("trace.txt").expect("Can't create
    // file")); env_logger::Builder::new()
    //     .target(env_logger::Target::Pipe(target))
    //     .filter(None, log::LevelFilter::Info)
    //     .init();

    // let args = Opt::from_args();

    // Bump the memlock rlimit. This is needed for older kernels that don't
    // use the new memcg based accounting, see https://lwn.net/Articles/837122/
    let rlim = libc::rlimit {
        rlim_cur: libc::RLIM_INFINITY,
        rlim_max: libc::RLIM_INFINITY,
    };
    let ret = unsafe { libc::setrlimit(libc::RLIMIT_MEMLOCK, &rlim) };
    if ret != 0 {
        log::debug!("remove limit on locked memory failed, ret is: {}", ret);
    }

    // Include the compiled BPF program in the binary.
    //
    let mut ebpf = Ebpf::load(include_bytes_aligned!(
        "../../target/bpfel-unknown-none/release/fdtrace"
    ))?;
    EbpfLogger::init(&mut ebpf)?;

    // Note: this name has to be the same as in the ebpf program.
    let program: &mut TracePoint = ebpf.program_mut("fdtrace").unwrap().try_into()?;
    program.load()?;
    program.attach("raw_syscalls", "sys_enter")?;

    //
    //

    let (tx, mut rx) = tokio::sync::mpsc::channel(1024);
    let mut events = AsyncPerfEventArray::try_from(ebpf.take_map("EVENTS").unwrap())?;

    for cpu_id in online_cpus().map_err(|(_, e)| e)? {
        // Process each perf buffer in a separate task.
        //
        // TODO: Set max entries for each CPU!!
        let mut buf = events.open(cpu_id, None)?;
        let tx = tx.clone();

        task::spawn(async move {
            // TODO: Why multiple buffers?
            let mut buffers = (0..10)
                .map(|_| BytesMut::with_capacity(1024))
                .collect::<Vec<_>>();

            loop {
                let events = buf.read_events(&mut buffers).await.unwrap();

                for buf in &buffers[..events.read] {
                    let data = SyscallLog::from_bytes(buf).unwrap();
                    tx.send(data).await.unwrap();
                }
            }
        });
    }

    task::spawn(async move {
        while let Some(data) = rx.recv().await {
            // Test with `watch ls`
            if !data.cmd().contains("watch\0") {
                continue;
            }

            // if data.pid == 472162 {
            //     log::info!("{:?}", data);
            // }
        }
    });

    let ctrl_c = signal::ctrl_c();
    println!("Waiting for Ctrl-C...");
    ctrl_c.await?;
    println!("Exiting...");

    Ok(())
}
