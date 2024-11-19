use super::file::{FileInfo, FileSession};
use crate::{
    analysis::{file::FileEvent, utils},
    syscall::{tid_t, RawSyscall, Syscall},
};
use itertools::Itertools;
use std::collections::HashMap;

const FD_STDIN: i32 = 0;
const FD_STDOUT: i32 = 1;
const FD_STDERR: i32 = 2;

#[derive(Debug)]
#[cfg_attr(test, derive(serde::Serialize))]
pub struct ThreadAnalysis {
    tid: tid_t,
    files: HashMap<String, FileInfo>,
}

impl ThreadAnalysis {
    pub fn new(tid: tid_t, syscalls: &[Syscall]) -> Self {
        log::info!("Thread {tid} got {{syscalls.len()}} syscalls");

        let mut files = HashMap::new();

        // Session is open until we get a `Close` syscall
        // TODO: Allow multiple sessions at the same time
        let mut cur_session = None;

        let mut iter = syscalls.iter().multipeek();
        while let Some(call) = iter.next() {
            assert_eq!(tid, call.tid);

            match &call.raw {
                RawSyscall::OpenAt { path, .. } | RawSyscall::Open { path, .. } => {
                    let Some(RawSyscall::OpenExit { fd } | RawSyscall::OpenAtExit { fd }) =
                        iter.peek().map(|s| &s.raw)
                    else {
                        // Sometimes we don't directly get a exit
                        //
                        // Example:
                        // openat;-100;/target/...;524288
                        // openat;-1059213040;;-1059213040
                        //
                        log::warn!("Syscall not followed by exit: {call:?}");
                        continue;
                    };

                    cur_session = Some((
                        fd,
                        path.clone(),
                        FileSession {
                            open_ts: call.ts,
                            ..Default::default()
                        },
                    ));
                    log::debug!("Created a new session for {path}");
                }
                RawSyscall::Read { fd: read_fd, .. } => {
                    if *read_fd == FD_STDIN {
                        continue;
                    }

                    let (open_fd, _, session) = cur_session
                        .as_mut()
                        .unwrap_or_else(|| panic!("Can't read without open: {call:?}"));
                    assert_eq!(*open_fd, read_fd, "Fd doesn't match: {call:?}");

                    let (end_ts, read) = match iter.peek().map(|s| (s.ts, &s.raw)) {
                        Some((end_ts, RawSyscall::ReadExit { read })) => (end_ts, read),
                        _ => {
                            // In some cases, the exit syscall is delayed by one
                            // syscall. We can check the next syscall to see if it's
                            // the exit syscall.
                            //
                            // 0;92062;92062;read;5;4096
                            // 1;92062;92062;read;-1050723616;18446631905292925664
                            // 3;92062;92062;read_exit;4096
                            // 4;92062;92062;read_exit;-1050734920
                            //
                            let Some((end_ts, RawSyscall::ReadExit { read })) =
                                iter.peek().map(|s| (s.ts, &s.raw))
                            else {
                                panic!("Read syscall not followed by read exit: {call:?}")
                            };

                            (end_ts, read)
                        }
                    };

                    // Only record if read was successful.
                    // See: https://man7.org/linux/man-pages/man2/read.2.html
                    //
                    if read != &-1 {
                        session.events.push(FileEvent::Read {
                            bytes: *read as usize,
                            start_ts: call.ts,
                            end_ts,
                        });
                        log::debug!("Read {read} bytes from {{session.path}}");
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
                        log::debug!("Wrote {written} bytes to {{session.path}}");
                    }
                }
                RawSyscall::Close { fd: close_fd } => {
                    if *close_fd == FD_STDIN || *close_fd == FD_STDOUT || *close_fd == FD_STDERR {
                        continue;
                    }

                    let Some((open_fd, path, mut session)) = cur_session.take() else {
                        log::warn!("Close without open: {call:?}");
                        continue;
                    };
                    assert_eq!(open_fd, close_fd, "{call:?}");
                    log::debug!("Closed {path}");

                    let file_info = files.entry(path).or_insert(FileInfo::default());
                    session.close_ts = call.ts;
                    file_info.sessions.push(session);
                }

                _ => {}
            }
        }

        Self { tid, files }
    }

    pub fn print_result(&self) {
        use termimad::print_inline as mdprintln;

        mdprintln(&format!("\n# **Thread: {}**\n\n", self.tid));
        for (path, file_info) in &self.files {
            mdprintln(&format!("\n## File: **{}**\n\n", path));
            println!("Opened: {} times", file_info.sessions.len());

            let total_duration = utils::ns_to_ms(
                file_info.sessions.iter().map(|s| s.duration()).sum::<u64>() as f64,
            );
            let avg_duration = total_duration / file_info.sessions.len() as f64;
            mdprintln(&format!("Total duration: {:.2} ms\n", total_duration));
            mdprintln(&format!("Avg session duration: {:.2} ms\n", avg_duration));

            // Sessions
            //
            println!();
            for (i, session) in file_info.sessions.iter().enumerate() {
                mdprintln(&format!(
                    "**Session {}** took {:.2}ms (idle for {:.2}ms)\n",
                    i + 1,
                    session.duration_ms(),
                    session.idle_time_ms()
                ));

                for (j, event) in session.events.iter().enumerate() {
                    match event {
                        FileEvent::Read { bytes, .. } => {
                            mdprintln(&format!("- **Event {}**: Read {} bytes\n", j + 1, bytes));
                        }
                        FileEvent::Write { bytes, .. } => {
                            mdprintln(&format!("- **Event {}**: Write {} bytes\n", j + 1, bytes));
                        }
                    }
                }
            }

            // IO Statistics
            //
            let (total_read, total_write) = file_info.total_bytes();
            let (avg_read, avg_write) = file_info.avg_size().unwrap_or_default();
            let (max_read, max_write) = file_info.max_size();

            let mut io_table = comfy_table::Table::new();
            io_table
                .set_header(vec!["", "Read", "Write"])
                .add_row(vec![
                    "Total",
                    &total_read.to_string(),
                    &total_write.to_string(),
                ])
                .add_row(vec![
                    "Average",
                    &avg_read.to_string(),
                    &avg_write.to_string(),
                ])
                .add_row(vec!["Max", &max_read.to_string(), &max_write.to_string()]);

            // mdprintln("\n### **IO Statistics**:\n\n");
            println!();
            println!("{io_table}");
        }
    }
}
