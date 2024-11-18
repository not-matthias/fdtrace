use crate::syscall::Syscall;
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
        let parse_pid = |line: &str| {
            line.split(';')
                .next()
                .and_then(|p: &str| p.parse::<u32>().ok())
                .unwrap_or_default()
        };

        let mut target_pid = None;
        let mut syscalls = Vec::new();
        for line in trace.lines() {
            let Some(target_pid) = target_pid else {
                // The output contains many other processes logs as well, which is not what we
                // want. We need to find the 'execve' syscall to find the process id of our
                // target process.
                //
                if line.contains("execve") {
                    target_pid = Some(parse_pid(line));
                }

                continue;
            };

            // After we have our target_pid, we can filter out all the other logs that
            // aren't related to this process.
            //
            if parse_pid(line) != target_pid {
                continue;
            }

            let parts = line.split(';').skip(1).collect::<Vec<_>>().join(";");

            if let Some(syscall) = Syscall::from_parts(&parts) {
                syscalls.push(syscall);
            } else {
                // TODO: Return as Result<_, _>
                log::error!("Failed to parse syscall: {}", parts);
            }
        }

        Ok(Self { syscalls })
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

#[cfg(test)]
mod tests {
    use super::*;

    // Run with: sudo cargo t
    #[test]
    fn test_trace_ls() {
        let tracer =
            BpfTracer::trace("/nix/store/sbxy42ph4gjlg567vaz1kihmgiwqa5dh-system-path/bin/ls")
                .unwrap();
        assert_ne!(tracer.syscalls.len(), 0);
    }
}
