use crate::syscall::{tid_t, Syscall};
use itertools::Itertools;
use std::collections::HashMap;
use thread::ThreadAnalysis;

pub mod file;
pub mod thread;
mod utils;

#[derive(Debug)]
#[cfg_attr(test, derive(serde::Serialize))]
pub struct Analysis {
    threads: HashMap<tid_t, ThreadAnalysis>,
}

impl Analysis {
    pub fn new(syscalls: Vec<Syscall>) -> Self {
        let threads = syscalls
            .into_iter()
            .into_group_map_by(|s| s.tid)
            .into_iter()
            .map(|(tid, syscalls)| (tid, ThreadAnalysis::new(tid, &syscalls)))
            .collect();

        Self { threads }
    }

    pub fn print_result(&self) {
        for (_, thread) in &self.threads {
            thread.print_result();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tracer::BpfTracer;

    #[test]
    fn test_analysis_0() {
        let raw_trace = include_str!("../../data/raw_trace_0.txt");
        let syscalls = BpfTracer::parse_trace(raw_trace).unwrap();
        let analysis = Analysis::new(syscalls);

        insta::with_settings!({sort_maps => true}, {
            insta::assert_json_snapshot!(analysis);
        });
    }

    #[test]
    fn test_analysis_1() {
        let raw_trace = include_str!("../../data/raw_trace_1.txt");
        let syscalls = BpfTracer::parse_trace(raw_trace).unwrap();
        let analysis = Analysis::new(syscalls);

        insta::with_settings!({sort_maps => true}, {
            insta::assert_json_snapshot!(analysis);
        });
    }

    #[test]
    fn test_analysis_2() {
        // This has the following edge case: Read syscalls is traced twice and then the
        // exit will be traced.
        //
        //  17332481476260;92062;92062;read;5;4096
        // 17332481478529;92062;92062;read;-1050723616;18446631905292925664
        // 17332481480112;92062;92062;read_exit;4096
        // 17332481480279;92062;92062;read_exit;-1050734920
        //
        let raw_trace = include_str!("../../data/raw_trace_2.txt");
        let syscalls = BpfTracer::parse_trace(raw_trace).unwrap();
        let analysis = Analysis::new(syscalls);

        insta::with_settings!({sort_maps => true}, {
            insta::assert_json_snapshot!(analysis);
        });
    }
}
