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
    fn test_analyze_multisession() {
        let raw_trace = include_str!("../../data/multisession.txt");
        let syscalls = BpfTracer::parse_trace(raw_trace).unwrap();
        let analysis = Analysis::new(syscalls);

        insta::with_settings!({sort_maps => true}, {
            insta::assert_json_snapshot!(analysis);
        });
    }

    #[test]
    fn test_analyze_threaded() {
        let raw_trace = include_str!("../../data/threaded.txt");
        let syscalls = BpfTracer::parse_trace(raw_trace).unwrap();
        let analysis = Analysis::new(syscalls);

        insta::with_settings!({sort_maps => true}, {
            insta::assert_json_snapshot!(analysis);
        });
    }
}
