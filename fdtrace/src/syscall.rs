use std::ops::{Deref, DerefMut};

#[allow(non_camel_case_types)]
pub type pid_t = i32;
#[allow(non_camel_case_types)]
pub type tid_t = i32;

#[allow(non_camel_case_types)]
pub type fd_t = u64;

#[derive(Debug, PartialEq)]
#[cfg_attr(test, derive(serde::Serialize))]
pub struct Syscall {
    pub ts: u64,
    pub pid: pid_t,
    pub tid: tid_t,
    pub raw: RawSyscall,
}

impl Syscall {
    pub fn from_parts(data: &str) -> Option<Self> {
        let parts = data.split(";");
        Self::from_parts_iter(parts)
    }

    pub fn from_parts_iter<'a>(mut parts: impl Iterator<Item = &'a str>) -> Option<Self> {
        Some(Self {
            ts: parts.next()?.parse().ok()?,
            pid: parts.next()?.parse().ok()?,
            tid: parts.next()?.parse().ok()?,
            raw: RawSyscall::from_parts_iter(parts)?,
        })
    }
}

impl Deref for Syscall {
    type Target = RawSyscall;

    fn deref(&self) -> &Self::Target {
        &self.raw
    }
}

impl DerefMut for Syscall {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.raw
    }
}

/// # Covered syscalls
///
/// - File creation and opening: open, openat.
/// - File descriptor operations: close, read, write.
/// - File removal: unlink, unlinkat, rmdir.
/// - File renaming: rename, renameat.
/// - Directory creation: mkdir, mkdirat.
#[derive(Debug, PartialEq)]
#[cfg_attr(test, derive(serde::Serialize))]
#[rustfmt::skip]
pub enum RawSyscall {
    Execve { path: String },

    Open { path: String, flags: u64, mode: u64 },
    OpenExit { ret: i64 },

    OpenAt { dirfd: fd_t, path: String, flags: u64 },
    OpenAtExit { ret: i64 },

    Close { fd: fd_t },
    CloseExit { ret: i64 },

    Read { fd: fd_t, count: usize },
    ReadExit { read: i64 },

    Write { fd: fd_t, count: usize },
    WriteExit { written: i64 },
}

impl RawSyscall {
    pub fn from_parts(data: &str) -> Option<Self> {
        let parts = data.split(";");
        Self::from_parts_iter(parts)
    }

    /// Parses the ';' separated syscall (e.g. `read;42;42`)
    pub fn from_parts_iter<'a>(mut parts: impl Iterator<Item = &'a str>) -> Option<Self> {
        macro_rules! parse_syscall {
            ($syscall:ident, $($field:ident),*) => {
                Some(RawSyscall::$syscall {
                    $($field: parts.next()?.parse().ok()?,)*
                })
            };
        }

        match parts.next()? {
            "execve" => parse_syscall!(Execve, path),

            "open" => parse_syscall!(Open, path, flags, mode),
            "open_exit" => parse_syscall!(OpenExit, ret),

            "openat" => parse_syscall!(OpenAt, dirfd, path, flags),
            "openat_exit" => parse_syscall!(OpenAtExit, ret),

            "close" => parse_syscall!(Close, fd),
            "close_exit" => parse_syscall!(CloseExit, ret),

            "read" => parse_syscall!(Read, fd, count),
            "read_exit" => parse_syscall!(ReadExit, read),

            "write" => parse_syscall!(Write, fd, count),
            "write_exit" => parse_syscall!(WriteExit, written),

            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tracer::BpfTracer;

    #[test]
    fn test_parse_threaded() {
        let raw_trace = include_str!("../data/threaded.txt");
        let syscalls = BpfTracer::parse_trace(&raw_trace).unwrap();

        insta::assert_json_snapshot!(syscalls);
    }

    #[test]
    fn test_parse_multisession() {
        let raw_trace = include_str!("../data/multisession.txt");
        let syscalls = BpfTracer::parse_trace(&raw_trace).unwrap();

        insta::assert_json_snapshot!(syscalls);
    }

    #[test]
    fn test_parse_read_exit() {
        let parts = "read_exit;832";
        let syscall = RawSyscall::from_parts(parts).unwrap();
        assert_eq!(syscall, RawSyscall::ReadExit { read: 832 });
    }

    #[test]
    fn test_parse_close() {
        let parts =
            "20429708185183;105898;105898;openat;18446631905284431216;;18446631905284431216";
        let syscall = Syscall::from_parts(parts).unwrap();
        assert_eq!(syscall.ts, 18836222727359);
        assert_eq!(syscall.pid, 99220);
        assert_eq!(syscall.tid, 99220);
        assert_eq!(
            syscall.raw,
            RawSyscall::Close {
                fd: 18446631905284438752
            }
        );
    }
}
