use super::utils;

#[derive(Debug, Default)]
#[cfg_attr(test, derive(serde::Serialize))]
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
#[cfg_attr(test, derive(serde::Serialize))]
pub struct FileSession {
    pub events: Vec<FileEvent>,
    pub open_ts: u64,
    pub close_ts: u64,
}

// Temporal
impl FileSession {
    pub const fn duration(&self) -> u64 {
        self.close_ts - self.open_ts
    }

    pub const fn duration_ms(&self) -> f64 {
        utils::ns_to_ms(self.duration() as f64)
    }

    pub fn idle_time_ms(&self) -> f64 {
        let mut total_idle = 0;
        let mut last_end = self.open_ts;

        for event in &self.events {
            let event_start = event.start_ts();
            total_idle += event_start - last_end;
            last_end = event.end_ts();
        }

        utils::ns_to_ms(total_idle as f64)
    }
}

#[derive(Debug)]
#[cfg_attr(test, derive(serde::Serialize))]
pub enum FileEvent {
    Read {
        bytes: usize,
        start_ts: u64,
        end_ts: u64,
    },
    Write {
        bytes: usize,
        start_ts: u64,
        end_ts: u64,
    },
}

impl FileEvent {
    pub const fn start_ts(&self) -> u64 {
        match self {
            FileEvent::Read { start_ts, .. } => *start_ts,
            FileEvent::Write { start_ts, .. } => *start_ts,
        }
    }

    pub const fn end_ts(&self) -> u64 {
        match self {
            FileEvent::Read { end_ts, .. } => *end_ts,
            FileEvent::Write { end_ts, .. } => *end_ts,
        }
    }
}
