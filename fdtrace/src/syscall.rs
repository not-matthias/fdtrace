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
pub enum Syscall {
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

    Unlink { pathname: String },
    UnlinkAt { dirfd: i32, pathname: String, flags: i32 },
    Rename { oldname: String, newname: String },
    RenameAt { olddfd: i32, oldname: String, newdfd: i32, newname: String },
    Mkdir { pathname: String, mode: i32 },
    MkdirAt { dirfd: i32, pathname: String, mode: i32 },
    Rmdir { pathname: String },
}

impl Syscall {
    pub fn from_parts(data: &str) -> Option<Self> {
        let parts = data.split(";").into_iter();
        Self::from_parts_iter(parts)
    }

    /// Parses the ';' separated syscall (e.g. `read;42;42`)
    pub fn from_parts_iter<'a>(mut parts: impl Iterator<Item = &'a str>) -> Option<Self> {
        macro_rules! parse_syscall {
            ($syscall:ident, $($field:ident),*) => {
                Some(Syscall::$syscall {
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
            "unlink" => parse_syscall!(Unlink, pathname),
            "unlinkat" => parse_syscall!(UnlinkAt, dirfd, pathname, flags),
            "rename" => parse_syscall!(Rename, oldname, newname),
            "renameat" => parse_syscall!(RenameAt, olddfd, oldname, newdfd, newname),
            "mkdir" => parse_syscall!(Mkdir, pathname, mode),
            "mkdirat" => parse_syscall!(MkdirAt, dirfd, pathname, mode),
            "rmdir" => parse_syscall!(Rmdir, pathname),
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
        let syscall = Syscall::from_parts(parts).unwrap();
        assert_eq!(syscall, Syscall::ReadExit { read: 832 });
    }
}
