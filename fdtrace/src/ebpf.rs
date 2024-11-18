use aya::{
    include_bytes_aligned, maps::AsyncPerfEventArray, programs::TracePoint, util::online_cpus, Ebpf,
};
use aya_log::EbpfLogger;
use bytes::BytesMut;
use fdtrace_common::SyscallLog;
use tokio::{signal, task};


// https://github.com/aurae-runtime/aurae/blob/1d62cb044fc973b3b8006466c31ef3f35ecbca63/auraed/src/ebpf/tracepoint/tracepoint_program.rs


pub async fn trace_pid(pid: u32) {

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
                // if !data.cmd().contains("watch\0") {
                //     continue;
                // }

                // log::info!("{:?}", data);
            }
        });

        let ctrl_c = signal::ctrl_c();
        println!("Waiting for Ctrl-C...");
        ctrl_c.await?;
        println!("Exiting...");

}
