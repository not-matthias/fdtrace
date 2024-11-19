use super::file::{FileInfo, FileSession};
use crate::{
    analysis::{file::FileEvent, utils},
    syscall::{fd_t, tid_t, RawSyscall, Syscall},
};
use itertools::Itertools;
use std::collections::HashMap;

#[derive(Debug)]
#[cfg_attr(test, derive(serde::Serialize))]
pub struct ThreadAnalysis {
    tid: tid_t,
    files: HashMap<String, FileInfo>,
}

impl ThreadAnalysis {
    pub fn new(tid: tid_t, syscalls: &[Syscall]) -> Self {
        log::info!("Thread {tid} got {} syscalls", syscalls.len());

        let mut files = HashMap::new();

        // All the current sessions. A new session is created when the file is opened,
        // and is removed from this list and added to `files` when the file is closed.
        //
        let mut cur_sessions = {
            let mut map = HashMap::new();

            // Add the default sessions for stdin, stdout, and stderr
            map.insert(0, FileSession::new("/dev/stdin"));
            map.insert(1, FileSession::new("/dev/stdout"));
            map.insert(2, FileSession::new("/dev/stderr"));

            map
        };

        let mut iter = syscalls.iter().multipeek();
        while let Some(call) = iter.next() {
            assert_eq!(tid, call.tid);

            match &call.raw {
                RawSyscall::OpenAt { path, .. } | RawSyscall::Open { path, .. } => {
                    let Some(RawSyscall::OpenExit { ret } | RawSyscall::OpenAtExit { ret }) =
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

                    if *ret != -1 {
                        cur_sessions.insert(
                            *ret as fd_t,
                            FileSession {
                                path: path.clone(),
                                open_ts: call.ts,
                                ..Default::default()
                            },
                        );
                        log::debug!("Created a new session for {path}");
                    }
                }
                RawSyscall::Read { fd: read_fd, .. } => {
                    let cur_session = cur_sessions
                        .get_mut(read_fd)
                        .unwrap_or_else(|| panic!("Can't read without open: {call:?}"));

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

                    // Only record if read was successful
                    // - 0 = EOF
                    // - -1 = error
                    //
                    if read > &0 {
                        cur_session.events.push(FileEvent::Read {
                            bytes: *read as usize,
                            start_ts: call.ts,
                            end_ts,
                        });
                        log::debug!("Read {read} bytes from {}", cur_session.path);
                    }
                }
                RawSyscall::Write { fd, .. } => {
                    let cur_session = cur_sessions
                        .get_mut(fd)
                        .unwrap_or_else(|| panic!("Can't write without open: {call:?}"));

                    let Some((end_ts, RawSyscall::WriteExit { written })) =
                        iter.peek().map(|s| (s.ts, &s.raw))
                    else {
                        panic!("Write syscall not followed by write exit")
                    };

                    // Only record if write was successful.
                    // - 0 = nothing was written
                    // - -1 = error
                    //
                    if written > &-0 {
                        cur_session.events.push(FileEvent::Write {
                            bytes: *written as usize,
                            start_ts: call.ts,
                            end_ts,
                        });
                        log::debug!("Wrote {written} bytes to {{session.path}}");
                    }
                }
                RawSyscall::Close { fd: close_fd } => {
                    let mut cur_session = cur_sessions
                        .remove(close_fd)
                        .unwrap_or_else(|| panic!("Close without open: {call:?}"));
                    log::debug!("Closed {{cur_session.path}}");

                    let file_info = files
                        .entry(cur_session.path.clone())
                        .or_insert(FileInfo::default());
                    cur_session.close_ts = call.ts;
                    file_info.sessions.push(cur_session);
                }

                _ => {}
            }
        }

        // Close the stdin, stdout, and stderr sessions
        //
        #[cfg(feature = "trace-stdfd")]
        for fd in 0..3 {
            let mut cur_session = cur_sessions.remove(&fd).unwrap();
            cur_session.open_ts = syscalls.first().unwrap().ts;
            cur_session.close_ts = syscalls.last().unwrap().ts;

            let file_info = files
                .entry(cur_session.path.clone())
                .or_insert(Default::default());
            file_info.sessions.push(cur_session);
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
                    "**Session {}** was open for {:.2}ms (idle for {:.2}ms)\n",
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
