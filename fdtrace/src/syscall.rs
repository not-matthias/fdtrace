use std::ops::{Deref, DerefMut};

#[allow(non_camel_case_types)]
pub type pid_t = usize;
#[allow(non_camel_case_types)]
pub type tid_t = i32;

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
    Open { path: String, flags: i32 },
    OpenExit { fd: i32 },
    OpenAt { dirfd: i32, path: String, flags: i32 },
    OpenAtExit { fd: i32 },
    Close { fd: i32 },
    CloseExit { ret: i32 },
    Read { fd: i32, count: usize },
    ReadExit { read: i32 },
    Write { fd: i32, count: usize },
    WriteExit { written: i32 },
    Execve { path: String },
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
            "open" => parse_syscall!(Open, path, flags),
            "open_exit" => parse_syscall!(OpenExit, fd),
            "openat" => parse_syscall!(OpenAt, dirfd, path, flags),
            "openat_exit" => parse_syscall!(OpenAtExit, fd),
            "close" => parse_syscall!(Close, fd),
            "close_exit" => parse_syscall!(CloseExit, ret),
            "read" => parse_syscall!(Read, fd, count),
            "read_exit" => parse_syscall!(ReadExit, read),
            "write" => parse_syscall!(Write, fd, count),
            "write_exit" => parse_syscall!(WriteExit, written),
            "execve" => parse_syscall!(Execve, path),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tracer::BpfTracer;

    #[test]
    fn test_parse_trace_0() {
        let raw_trace = include_str!("../data/raw_trace_0.txt");
        let syscalls = BpfTracer::parse_trace(&raw_trace).unwrap();

        insta::assert_json_snapshot!(syscalls);
    }

    #[test]
    fn test_parse_trace_1() {
        let raw_trace = include_str!("../data/raw_trace_1.txt");
        let syscalls = BpfTracer::parse_trace(&raw_trace).unwrap();

        insta::assert_json_snapshot!(syscalls);
    }

    #[test]
    fn test_parse_openat_with_syscall() {
        let parts = "12207973532783;50980;50981;openat;-100;/etc/hosts;524288";
        let syscall = Syscall::from_parts(parts).unwrap();
        assert_eq!(syscall.ts, 12207973532783);
        assert_eq!(syscall.pid, 50980);
        assert_eq!(syscall.tid, 50981);
        assert_eq!(
            syscall.raw,
            RawSyscall::OpenAt {
                dirfd: -100,
                path: "/etc/hosts".to_string(),
                flags: 524288
            }
        );
    }

    #[test]
    fn test_parse_read_exit() {
        let parts = "read_exit;832";
        let syscall = RawSyscall::from_parts(parts).unwrap();
        assert_eq!(syscall, RawSyscall::ReadExit { read: 832 });
    }
}
