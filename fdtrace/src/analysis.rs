pub fn summary(&self) {
    // Aggregate Statistics:
    // - Calculate and display aggregate statistics for each file, such as:
    //      - Total bytes read/written.
    //      - Average size of read/write operations.
    //      - Maximum and minimum operation sizes
    //
    use std::collections::HashMap;

    let mut total_bytes_read = 0;
    let mut total_bytes_written = 0;
    let mut read_count = 0;
    let mut write_count = 0;
    let mut min_read_size = usize::MAX;
    let mut max_read_size = 0;
    let mut min_write_size = usize::MAX;
    let mut max_write_size = 0;

    // Track stats per file descriptor
    let mut fd_stats: HashMap<i32, (usize, usize)> = HashMap::new(); // (bytes_read, bytes_written)

    for syscall in &self.syscalls {
        match syscall {
            Syscall::Read { fd, count } => {
                total_bytes_read += count;
                read_count += 1;
                min_read_size = min_read_size.min(*count);
                max_read_size = max_read_size.max(*count);

                let entry = fd_stats.entry(*fd).or_insert((0, 0));
                entry.0 += count;
            }
            Syscall::Write { fd, count } => {
                total_bytes_written += count;
                write_count += 1;
                min_write_size = min_write_size.min(*count);
                max_write_size = max_write_size.max(*count);

                let entry = fd_stats.entry(*fd).or_insert((0, 0));
                entry.1 += count;
            }
            _ => {}
        }
    }

    println!("\nAggregate Statistics:");
    println!("Total bytes read: {}", total_bytes_read);
    println!("Total bytes written: {}", total_bytes_written);
    if read_count > 0 {
        println!("Average read size: {}", total_bytes_read / read_count);
        println!("Min read size: {}", min_read_size);
        println!("Max read size: {}", max_read_size);
    }
    if write_count > 0 {
        println!("Average write size: {}", total_bytes_written / write_count);
        println!("Min write size: {}", min_write_size);
        println!("Max write size: {}", max_write_size);
    }

    println!("\nPer File Descriptor Statistics:");
    for (fd, (bytes_read, bytes_written)) in fd_stats {
        println!(
            "FD {}: {} bytes read, {} bytes written",
            fd, bytes_read, bytes_written
        );
    }
}
