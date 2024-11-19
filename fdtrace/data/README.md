raw_trace_2: read without exit
thread 'analysis::tests::test_analysis_2' panicked at fdtrace/src/analysis/thread.rs:71:25:
Read syscall not followed by read exit: Syscall { ts: 17332481476260, pid: 92062, tid: 92062, raw: Read { fd: 5, count: 4096 } }
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

17332481476260;92062;92062;read;5;4096
17332481478529;92062;92062;read;-1050723616;18446631905292925664
17332481480112;92062;92062;read_exit;4096
17332481480279;92062;92062;read_exit;-1050734920
