use crate::syscall::Syscall;
use std::collections::HashMap;

const FD_STDIN: i32 = 0;
const FD_STDOUT: i32 = 1;
const FD_STDERR: i32 = 2;

#[derive(Debug, Default)]
pub struct FileInfo {
    pub sessions: Vec<FileSession>,
}

impl FileInfo {
    /// Returns the total number of read and write events.
    pub fn total_bytes(&self) -> (usize, usize) {
        let mut total_read_bytes = 0;
        let mut total_write_bytes = 0;

        for session in &self.sessions {
            for event in &session.events {
                match event {
                    FileEvent::Read(bytes) => {
                        total_read_bytes += *bytes;
                    }
                    FileEvent::Write(bytes) => {
                        total_write_bytes += *bytes;
                    }
                }
            }
        }

        (total_read_bytes, total_write_bytes)
    }

    /// Returns average read and write size.
    pub fn avg_size(&self) -> Option<(f64, f64)> {
        let mut total_read_bytes = 0_f64;
        let mut total_read_count = 0_usize;

        let mut total_write_bytes = 0_f64;
        let mut total_write_count = 0_usize;

        for session in &self.sessions {
            for event in &session.events {
                match event {
                    FileEvent::Read(bytes) => {
                        total_read_bytes += *bytes as f64;
                        total_read_count += 1;
                    }
                    FileEvent::Write(bytes) => {
                        total_write_bytes += *bytes as f64;
                        total_write_count += 1;
                    }
                }
            }
        }

        if total_read_count == 0 || total_write_count == 0 {
            return None;
        }

        Some((
            total_read_bytes / total_read_count as f64,
            total_write_bytes / total_write_count as f64,
        ))
    }

    /// Returns the maximum read and write size.
    pub fn max_size(&self) -> (usize, usize) {
        let mut max_read = 0;
        let mut max_write = 0;

        for session in &self.sessions {
            for event in &session.events {
                match event {
                    FileEvent::Read(bytes) => {
                        max_read = max_read.max(*bytes);
                    }
                    FileEvent::Write(bytes) => {
                        max_write = max_write.max(*bytes);
                    }
                }
            }
        }

        (max_read, max_write)
    }
}

#[derive(Debug, Default)]
pub struct FileSession {
    pub events: Vec<FileEvent>,
}

#[derive(Debug)]
pub enum FileEvent {
    Read(usize),
    Write(usize),
}

// File: /etc/passwd
//  Opened: 2 times
//  Sessions:
//      - Session 1:
//          * Event 1: Write 80 bytes
//          * Event 2: Read 30 bytes
//          * Event 3: Write 80 bytes
//      - Session 2:
//          * Event 1: Read 100 bytes
//          * Event 2: Write 50 bytes
//
// File: /var/log/syslog
//  Opened: 1 time
//  Sessions:
//      - Session 1:
//          * Event 1: Write 2048 bytes
//          * Event 2: Read 128 bytes
pub struct Agg {
    files: HashMap<String, FileInfo>,
}

impl Agg {
    pub fn analyze(syscalls: &[Syscall]) -> Self {
        let mut files = HashMap::new();

        // Session is open until we get a `Close` syscall
        //
        let mut cur_session = None;

        let mut iter = syscalls.iter().peekable();
        while let Some(call) = iter.next() {
            match call {
                Syscall::OpenAt { filename, .. } => {
                    let Some(Syscall::OpenAtExit { fd }) = iter.peek() else {
                        panic!("OpenAt syscall not followed by OpenAtExit")
                    };

                    cur_session = Some((fd, filename.clone(), FileSession::default()));
                }
                Syscall::Open { filename: path, .. } => {
                    let Some(Syscall::OpenExit { fd }) = iter.peek() else {
                        panic!("Open syscall not followed by open exit")
                    };

                    cur_session = Some((fd, path.clone(), FileSession::default()));
                }
                Syscall::Read { fd: read_fd, .. } => {
                    if *read_fd == FD_STDIN {
                        continue;
                    }

                    let (open_fd, _, session) =
                        cur_session.as_mut().expect("Can't read without open");
                    assert_eq!(*open_fd, read_fd);

                    let Some(Syscall::ReadExit { read }) = iter.peek() else {
                        panic!("Read syscall not followed by read exit")
                    };

                    // Return value of `read` syscall:
                    // 0 = EOF
                    // -1 = Error
                    // Otherwise, bytes read
                    // See: https://man7.org/linux/man-pages/man2/read.2.html
                    //
                    if read != &-1 {
                        session.events.push(FileEvent::Read(*read as usize));
                    }
                }
                Syscall::Write { fd: write_fd, .. } => {
                    if *write_fd == FD_STDOUT || *write_fd == FD_STDERR {
                        continue;
                    }

                    let (open_fd, _, session) =
                        cur_session.as_mut().expect("Can't write without open");
                    assert_eq!(*open_fd, write_fd);

                    let Some(Syscall::WriteExit { written }) = iter.peek() else {
                        panic!("Write syscall not followed by write exit")
                    };

                    if written != &-1 {
                        session.events.push(FileEvent::Write(*written as usize));
                    }
                }
                Syscall::Close { fd: close_fd } => {
                    if *close_fd == FD_STDIN || *close_fd == FD_STDOUT || *close_fd == FD_STDERR {
                        continue;
                    }

                    let (open_fd, path, session) =
                        cur_session.take().expect("Can't close without open");
                    assert_eq!(open_fd, close_fd);

                    let file_info = files.entry(path).or_insert(FileInfo::default());
                    file_info.sessions.push(session);
                }

                _ => {}
            }
        }

        Agg { files }
    }
}

impl core::fmt::Display for Agg {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        for (path, file_info) in &self.files {
            // Don't print files with 0 reads and writes
            let (total_read, total_write) = file_info.total_bytes();
            if total_read == 0 && total_write == 0 {
                continue;
            }

            writeln!(f, "File: {}", path)?;
            writeln!(f, " Opened: {} times", file_info.sessions.len())?;

            // Print the statistics
            writeln!(f, " Statistics:")?;
            writeln!(f, "  - Total read bytes: {}", total_read)?;
            writeln!(f, "  - Total write bytes: {}", total_write)?;

            if let Some((avg_read, avg_write)) = file_info.avg_size() {
                writeln!(f, "  - Average read size: {:.2} bytes", avg_read)?;
                writeln!(f, "  - Average write size: {:.2} bytes", avg_write)?;
            }
            let (max_read, max_write) = file_info.max_size();
            writeln!(f, "  - Maximum read size: {} bytes", max_read)?;
            writeln!(f, "  - Maximum write size: {} bytes", max_write)?;

            writeln!(f, " Sessions:")?;
            for (i, session) in file_info.sessions.iter().enumerate() {
                writeln!(f, "  - Session {}:", i)?;
                for (j, event) in session.events.iter().enumerate() {
                    match event {
                        FileEvent::Read(bytes) => {
                            writeln!(f, "   * Event {}: Read {} bytes", j + 1, bytes)?;
                        }
                        FileEvent::Write(bytes) => {
                            writeln!(f, "   * Event {}: Write {} bytes", j + 1, bytes)?;
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}
