use crate::syscall::{RawSyscall, Syscall};
use std::path::Path;
use tempfile::NamedTempFile;

pub struct BpfTracer {
    syscalls: Vec<Syscall>,
}

impl BpfTracer {
    pub fn trace(program: &Path) -> anyhow::Result<Self> {
        let tmpfile = NamedTempFile::new()?;
        let cmd = std::process::Command::new("bpftrace")
            .arg("-c")
            .arg(program)
            .arg("-o")
            .arg(tmpfile.path())
            // TODO: Fix this path
            .arg("scripts/fdtrace.bt")
            .output()?;

        if !cmd.status.success() {
            let error = String::from_utf8_lossy(&cmd.stderr);
            return Err(anyhow::anyhow!("{error}"));
        }

        let output = std::fs::read_to_string(tmpfile)?;
        Self::parse_trace(&output)
    }

    pub fn parse_trace(trace: &str) -> anyhow::Result<Self> {
        let mut target_pid = None;

        let mut syscalls = Vec::new();
        for line in trace.lines().skip(1) {
            let syscall = Syscall::from_parts(line).unwrap();

            // The output contains many other processes logs as well, which is not what we
            // want. We need to find the 'execve' syscall to find the process id of our
            // target process.
            //
            let Some(target_pid) = target_pid else {
                if let RawSyscall::Execve { path } = &syscall.raw {
                    log::info!("Target process: {:?}", path);
                    target_pid = Some(syscall.pid);
                }
                continue;
            };

            // After we have our target_pid, we can filter out all the other logs that
            // aren't related to this process.
            //
            if syscall.pid != target_pid {
                continue;
            }

            syscalls.push(syscall);
        }

        Ok(Self { syscalls })
    }

    pub fn syscalls(&self) -> &[Syscall] {
        &self.syscalls
    }

    /// Prints the syscalls to a file
    pub fn print_to_file(&self, file: &str) {
        use std::io::Write;

        let mut file = std::fs::File::create(file).unwrap();
        for syscall in &self.syscalls {
            writeln!(file, "{:?}", syscall).unwrap();
        }
    }

    /// Prints the syscalls to stdout
    pub fn debug_print(&self) {
        for syscall in &self.syscalls {
            println!("{:?}", syscall);
        }
    }
}
