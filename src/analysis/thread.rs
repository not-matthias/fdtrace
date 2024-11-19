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

                RawSyscall::Read { fd, .. } | RawSyscall::Write { fd, .. } => {
                    let is_read = matches!(call.raw, RawSyscall::Read { .. });

                    let Some(cur_session) = cur_sessions.get_mut(fd) else {
                        log::warn!("RW without open: {call:?}");
                        continue;
                    };

                    macro_rules! peek_peek_exit {
                        ($name:path) => {{
                            let Some((end_ts, $name { count })) = iter
                                .peek()
                                .map(|s| (s.ts, &s.raw))
                                .or_else(|| iter.peek().map(|s| (s.ts, &s.raw)))
                            else {
                                // FIXME: We potentially lost a read event here. But it's still
                                // better to continue instead of panicking.
                                log::warn!("Read syscall not followed by read exit: {call:?}");
                                continue;
                            };

                            (end_ts, count)
                        }};
                    }

                    let (end_ts, count) = if is_read {
                        peek_peek_exit!(RawSyscall::ReadExit)
                    } else {
                        peek_peek_exit!(RawSyscall::WriteExit)
                    };

                    // Only record if read was successful
                    // - 0 = EOF
                    // - -1 = error
                    //
                    if count > &0 {
                        if is_read {
                            cur_session.events.push(FileEvent::Read {
                                bytes: *count as usize,
                                start_ts: call.ts,
                                end_ts,
                            });
                        } else {
                            cur_session.events.push(FileEvent::Write {
                                bytes: *count as usize,
                                start_ts: call.ts,
                                end_ts,
                            });
                        }
                    }
                }

                RawSyscall::Close { fd: close_fd } => {
                    let Some(mut cur_session) = cur_sessions.remove(close_fd) else {
                        // FIXME: More syscalls need to be traced to also catch this.
                        log::warn!("Close without open: {call:?}");
                        continue;
                    };
                    cur_session.close_ts = call.ts;
                    log::debug!("Closed {}", cur_session.path);

                    let file_info = files
                        .entry(cur_session.path.clone())
                        .or_insert(FileInfo::default());
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
