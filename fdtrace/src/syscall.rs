use std::ops::{Deref, DerefMut};

#[derive(Debug, PartialEq)]
pub struct Syscall {
    pub ts: i64,
    pub pid: i32,
    pub tid: i32,
    pub raw: RawSyscall,
}

impl Syscall {
    pub fn from_parts(data: &str) -> Option<Self> {
        let parts = data.split(";").into_iter();
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
#[rustfmt::skip]
pub enum RawSyscall {
    Open { filename: String, flags: i32 },
    OpenExit { fd: i32 },
    OpenAt { dirfd: i32, filename: String, flags: i32 },
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
        let parts = data.split(";").into_iter();
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
            "open" => parse_syscall!(Open, filename, flags),
            "open_exit" => parse_syscall!(OpenExit, fd),
            "openat" => parse_syscall!(OpenAt, dirfd, filename, flags),
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

    // TODO: add more tests

    #[test]
    fn test_parse_read_exit() {
        let parts = "read_exit;832";
        let syscall = RawSyscall::from_parts(parts).unwrap();
        assert_eq!(syscall, RawSyscall::ReadExit { read: 832 });
    }
}
