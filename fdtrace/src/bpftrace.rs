//! Small wrapper program around bpftrace.

// Trace by pid:
//  -p PID
// -e PROGRAM
// -c COMMAND

pub struct BpfTracer {
    syscalls: Vec<Syscall>,
}

impl BpfTracer {
    pub fn run(program: &str) -> Result<String, String> {
        let output = std::process::Command::new("bpftrace")
            .arg("-c")
            .arg(program)
            // TODO: Fix this path
            .arg("scripts/fdtrace.bt")
            .output()
            .map_err(|e| e.to_string())?;

        if !output.status.success() {
            return Err(String::from_utf8_lossy(&output.stderr).to_string());
        }

        String::from_utf8(output.stdout).map_err(|e| e.to_string())
    }

    pub fn new(program: &str) -> Self {
        let output = Self::run(program).unwrap();
        let syscalls: Vec<_> = output
            .split('|')
            .filter_map(|line| Syscall::from_parts(line))
            .collect();

        Self { syscalls }
    }

    /// Prints the syscalls to a file (for debugging purposes).
    pub fn print_to_file(&self, file: &str) {
        use std::io::Write;

        let mut file = std::fs::File::create(file).unwrap();
        for syscall in &self.syscalls {
            writeln!(file, "{:?}", syscall).unwrap();
        }
    }

    pub fn summary(&self) {
        //
        //
    }
}

/// # Covered syscalls
///
/// - File creation and opening: open, openat.
/// - File descriptor operations: close, read, write.
/// - File removal: unlink, unlinkat, rmdir.
/// - File renaming: rename, renameat.
/// - Directory creation: mkdir, mkdirat.
// TODO: Move this to the common crate
#[derive(Debug, PartialEq)]
#[rustfmt::skip]
enum Syscall {
    Open { filename: String, flags: i32 },
    OpenAt { dirfd: i32, filename: String, flags: i32 },
    Close { fd: i32 },
    Read { fd: i32, count: usize },
    Write { fd: i32, count: usize },
    Unlink { pathname: String },
    UnlinkAt { dirfd: i32, pathname: String, flags: i32 },
    Rename { oldname: String, newname: String },
    RenameAt { olddfd: i32, oldname: String, newdfd: i32, newname: String },
    Mkdir { pathname: String, mode: i32 },
    MkdirAt { dirfd: i32, pathname: String, mode: i32 },
    Rmdir { pathname: String },
}

impl Syscall {
    /// Parses the ';' separated syscall (e.g. `read;42;42`)
    fn from_parts(data: &str) -> Option<Self> {
        let parts: Vec<&str> = data.split(';').collect();

        macro_rules! parse_syscall {
            ($syscall:ident, $field:ident) => {
                Some(Syscall::$syscall {
                    $field: parts.get(1)?.parse().ok()?,
                })
            };

            ($syscall:ident, $field1:ident, $field2:ident) => {
                Some(Syscall::$syscall {
                    $field1: parts.get(1)?.parse().ok()?,
                    $field2: parts.get(2)?.parse().ok()?,
                })
            };

            ($syscall:ident, $field1:ident, $field2:ident, $field3:ident) => {
                Some(Syscall::$syscall {
                    $field1: parts.get(1)?.parse().ok()?,
                    $field2: parts.get(2)?.parse().ok()?,
                    $field3: parts.get(3)?.parse().ok()?,
                })
            };

            ($syscall:ident, $field1:ident, $field2:ident, $field3:ident, $field4:ident) => {
                Some(Syscall::$syscall {
                    $field1: parts.get(1)?.parse().ok()?,
                    $field2: parts.get(2)?.parse().ok()?,
                    $field3: parts.get(3)?.parse().ok()?,
                    $field4: parts.get(4)?.parse().ok()?,
                })
            };
        }

        match parts.get(0)? {
            &"open" => parse_syscall!(Open, filename, flags),
            &"openat" => parse_syscall!(OpenAt, dirfd, filename, flags),
            &"close" => parse_syscall!(Close, fd),
            &"read" => parse_syscall!(Read, fd, count),
            &"write" => parse_syscall!(Write, fd, count),
            &"unlink" => parse_syscall!(Unlink, pathname),
            &"unlinkat" => parse_syscall!(UnlinkAt, dirfd, pathname, flags),
            &"rename" => parse_syscall!(Rename, oldname, newname),
            &"renameat" => parse_syscall!(RenameAt, olddfd, oldname, newdfd, newname),
            &"mkdir" => parse_syscall!(Mkdir, pathname, mode),
            &"mkdirat" => parse_syscall!(MkdirAt, dirfd, pathname, mode),
            &"rmdir" => parse_syscall!(Rmdir, pathname),
            _ => None,
        }
    }

    #[deprecated]
    fn from_json(line: &str) -> Option<Self> {
        // Parse the JSON:
        // e.g. {"type": "printf", "data": "read;42;42"}
        //
        let json = json::parse(line).ok()?;
        let data = json["data"].as_str()?;

        Self::from_parts(data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trace_ls() {
        // sudo bpftrace -f json -c
        // /nix/store/sbxy42ph4gjlg567vaz1kihmgiwqa5dh-system-path/bin/ls
        // scripts/fdtrace.bt
        env_logger::init();

        let _tracer =
            BpfTracer::new("/nix/store/sbxy42ph4gjlg567vaz1kihmgiwqa5dh-system-path/bin/ls");

        log::info!("{:?}", _tracer.syscalls);
        eprintln!("{:?}", _tracer.syscalls);

        assert_ne!(_tracer.syscalls.len(), 0);
        panic!()
    }

    #[test]
    fn parse_read_normal() {
        let line = r#"{"type": "printf", "data": "read;944024616;18446638392688290856"}"#;
        assert_eq!(
            Syscall::from_json(line),
            Some(Syscall::Read {
                fd: 944024616,
                count: 18446638392688290856
            })
        );
    }

    #[test]
    fn parse_openat_normal() {
        let line = r#"{"type": "printf", "data": "openat;-100;/proc/8194/task/8243/task;591872"}"#;

        assert_eq!(
            Syscall::from_json(line),
            Some(Syscall::OpenAt {
                dirfd: -100,
                filename: "/proc/8194/task/8243/task".to_string(),
                flags: 591872,
            })
        );
    }

    #[test]
    fn parse_openat_missing_filename() {
        let line = r#"{"type": "printf", "data": "openat;-2133126240;;-2133126240"}"#;
        assert_eq!(
            Syscall::from_json(line),
            Some(Syscall::OpenAt {
                dirfd: -2133126240,
                filename: "".to_string(),
                flags: -2133126240,
            })
        );
    }
}
