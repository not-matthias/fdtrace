use crate::syscall::{RawSyscall, Syscall};
use std::collections::HashMap;

const FD_STDIN: i32 = 0;
const FD_STDOUT: i32 = 1;
const FD_STDERR: i32 = 2;

#[derive(Debug, Default)]
pub struct FileInfo {
    pub sessions: Vec<FileSession>,
}

// Aggregation
impl FileInfo {
    /// Returns the total number of read and write events.
    pub fn total_bytes(&self) -> (usize, usize) {
        let mut total_read_bytes = 0;
        let mut total_write_bytes = 0;

        for session in &self.sessions {
            for event in &session.events {
                match event {
                    FileEvent::Read { bytes, .. } => {
                        total_read_bytes += *bytes;
                    }
                    FileEvent::Write { bytes, .. } => {
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
                    FileEvent::Read { bytes, .. } => {
                        total_read_bytes += *bytes as f64;
                        total_read_count += 1;
                    }
                    FileEvent::Write { bytes, .. } => {
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
                    FileEvent::Read { bytes, .. } => {
                        max_read = max_read.max(*bytes);
                    }
                    FileEvent::Write { bytes, .. } => {
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
    pub open_ts: i64,
    pub close_ts: i64,
}

// Temporal
impl FileSession {
    pub const fn duration(&self) -> i64 {
        self.close_ts - self.open_ts
    }

    pub const fn duration_ms(&self) -> f64 {
        self.duration() as f64 / 1_000_000.0 // Convert to milliseconds
    }

    pub fn idle_time_ms(&self) -> f64 {
        // TODO: Verify this!!!
        let mut total_idle = 0_i64;
        let mut last_end = self.open_ts;

        for event in &self.events {
            let event_start = event.start_ts();
            total_idle += event_start - last_end;
            last_end = event.end_ts();
        }

        total_idle as f64 / 1_000_000.0 // Convert to milliseconds
    }
}

#[derive(Debug)]
pub enum FileEvent {
    Read {
        bytes: usize,
        start_ts: i64,
        end_ts: i64,
    },
    Write {
        bytes: usize,
        start_ts: i64,
        end_ts: i64,
    },
}

impl FileEvent {
    pub const fn start_ts(&self) -> i64 {
        match self {
            FileEvent::Read { start_ts, .. } => *start_ts,
            FileEvent::Write { start_ts, .. } => *start_ts,
        }
    }

    pub const fn end_ts(&self) -> i64 {
        match self {
            FileEvent::Read { end_ts, .. } => *end_ts,
            FileEvent::Write { end_ts, .. } => *end_ts,
        }
    }
}

#[derive(Debug)]
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
            match &call.raw {
                // TODO: Rename to path
                RawSyscall::OpenAt { filename: path, .. }
                | RawSyscall::Open { filename: path, .. } => {
                    let Some(RawSyscall::OpenExit { fd } | RawSyscall::OpenAtExit { fd }) =
                        iter.peek().map(|s| &s.raw)
                    else {
                        panic!("Syscall not followed by exit")
                    };

                    cur_session = Some((
                        fd,
                        path.clone(),
                        FileSession {
                            open_ts: call.ts,
                            ..Default::default()
                        },
                    ));
                }
                RawSyscall::Read { fd: read_fd, .. } => {
                    if *read_fd == FD_STDIN {
                        continue;
                    }

                    let (open_fd, _, session) =
                        cur_session.as_mut().expect("Can't read without open");
                    assert_eq!(*open_fd, read_fd);

                    let Some((end_ts, RawSyscall::ReadExit { read })) =
                        iter.peek().map(|s| (s.ts, &s.raw))
                    else {
                        panic!("Read syscall not followed by read exit")
                    };

                    // Return value of `read` syscall:
                    // 0 = EOF
                    // -1 = Error
                    // Otherwise, bytes read
                    // See: https://man7.org/linux/man-pages/man2/read.2.html
                    //
                    if read != &-1 {
                        session.events.push(FileEvent::Read {
                            bytes: *read as usize,
                            start_ts: call.ts,
                            end_ts,
                        });
                    }
                }
                RawSyscall::Write { fd: write_fd, .. } => {
                    if *write_fd == FD_STDOUT || *write_fd == FD_STDERR {
                        continue;
                    }

                    let (open_fd, _, session) =
                        cur_session.as_mut().expect("Can't write without open");
                    assert_eq!(*open_fd, write_fd);

                    let Some((end_ts, RawSyscall::WriteExit { written })) =
                        iter.peek().map(|s| (s.ts, &s.raw))
                    else {
                        panic!("Write syscall not followed by write exit")
                    };

                    if written != &-1 {
                        session.events.push(FileEvent::Write {
                            bytes: *written as usize,
                            start_ts: call.ts,
                            end_ts,
                        });
                    }
                }
                RawSyscall::Close { fd: close_fd } => {
                    if *close_fd == FD_STDIN || *close_fd == FD_STDOUT || *close_fd == FD_STDERR {
                        continue;
                    }

                    let (open_fd, path, mut session) =
                        cur_session.take().expect("Can't close without open");
                    assert_eq!(open_fd, close_fd);

                    session.close_ts = call.ts;

                    let file_info = files.entry(path).or_insert(FileInfo::default());
                    file_info.sessions.push(session);
                }

                _ => {}
            }
        }

        Agg { files }
    }

    pub fn print_result(&self) {
        for (path, file_info) in &self.files {
            let (total_read, total_write) = file_info.total_bytes();
            if total_read == 0 && total_write == 0 {
                continue;
            }

            println!("File: {}", path);
            println!("\tOpened: {} times", file_info.sessions.len());

            // IO Statistics
            //
            println!("\tIO Statistics:");
            println!("\t\t- Total read bytes: {}", total_read);
            println!("\t\t- Total write bytes: {}", total_write);

            if let Some((avg_read, avg_write)) = file_info.avg_size() {
                println!("\t\t- Average read size: {:.2} bytes", avg_read);
                println!("\t\t- Average write size: {:.2} bytes", avg_write);
            }

            let (max_read, max_write) = file_info.max_size();
            println!("\t\t- Maximum read size: {} bytes", max_read);
            println!("\t\t- Maximum write size: {} bytes", max_write);

            // Timing Statistics
            //
            println!("\tTiming Statistics:");

            let total_duration =
                file_info.sessions.iter().map(|s| s.duration()).sum::<i64>() as f64 / 1_000_000.0;
            let avg_duration = total_duration / file_info.sessions.len() as f64;

            println!("\t\t- Total duration: {:.2} ms", total_duration);
            println!("\t\t- Average session duration: {:.2} ms", avg_duration);

            // Sessions
            //
            println!("\tSessions:");
            for (i, session) in file_info.sessions.iter().enumerate() {
                println!(
                    "\t\t- Session {} (duration = {:.2}ms, idle = {:.2}ms):",
                    i + 1,
                    session.duration_ms(),
                    session.idle_time_ms()
                );

                for (j, event) in session.events.iter().enumerate() {
                    match event {
                        FileEvent::Read { bytes, .. } => {
                            println!("\t\t\t* Event {}: Read {} bytes", j + 1, bytes);
                        }
                        FileEvent::Write { bytes, .. } => {
                            println!("\t\t\t* Event {}: Write {} bytes", j + 1, bytes);
                        }
                    }
                }
            }
        }
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
                        FileEvent::Read { bytes, .. } => {
                            writeln!(f, "   * Event {}: Read {} bytes", j + 1, bytes)?;
                        }
                        FileEvent::Write { bytes, .. } => {
                            writeln!(f, "   * Event {}: Write {} bytes", j + 1, bytes)?;
                        }
                    }
                }
            }
        }

        Ok(())
    }
}
