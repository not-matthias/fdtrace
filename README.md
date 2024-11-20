# fdtrace

File system call tracer using using `bpftrace` and analysis.

## Prerequisites

- [Nightly Rust](https://www.rust-lang.org/tools/install)
- [bpftrace](https://bpftrace.org/): `sudo apt install -y bpftrace` for Ubuntu 19.04 and later
- Linux Kernel > 4.7 (for `bpftrace` tracepoint support)

Optional:
- [Devenv.sh](https://devenv.sh/) for a reproducible development environment
- [cargo-insta](https://crates.io/crates/cargo-insta): To review tests.

## Run

```bash
$ sudo -E cargo rr --help
$ sudo -E cargo rr --debug (whereis ls)
```

To run the example:
```bash
$ cargo br --example multisession
$ sudo -E cargo rr 'target/release/examples/multisession' --debug
```


## Debugging

Run your command with `strace` and compare the syscalls:
```bash
$ strace ls 2> strace.txt
```

Run fdtrace with `--debug` which creates `debug.txt`:
```bash
$ sudo -E cargo rr /run/current-system/sw/bin/ls --debug
```

Compare `debug.txt` and `strace.txt`.


## Example output

Trace the `multisession` example:
```
sudo target/release/fdtrace target/release/examples/multisession
```

The output:
```
# Thread: 1899

## File: /etc/hosts

Opened: 10 times
Total duration: 5001.70 ms
Avg session duration: 500.17 ms

Session 1 was open for 500.23ms (idle for 500.17ms)
- Event 1: Read 326 bytes
Session 2 was open for 500.12ms (idle for 0.00ms)
Session 3 was open for 500.20ms (idle for 500.14ms)
- Event 1: Read 326 bytes
Session 4 was open for 500.15ms (idle for 0.00ms)
Session 5 was open for 500.18ms (idle for 500.12ms)
- Event 1: Read 326 bytes
Session 6 was open for 500.14ms (idle for 0.00ms)
Session 7 was open for 500.23ms (idle for 500.16ms)
- Event 1: Read 326 bytes
Session 8 was open for 500.10ms (idle for 0.00ms)
Session 9 was open for 500.21ms (idle for 500.15ms)
- Event 1: Read 326 bytes
Session 10 was open for 500.14ms (idle for 0.00ms)

+---------+------+-------+
|         | Read | Write |
+========================+
| Total   | 1630 | 0     |
|---------+------+-------|
| Average | 0    | 0     |
|---------+------+-------|
| Max     | 326  | 0     |
+---------+------+-------+

# Thread: 1898


## File: /lib/x86_64-linux-gnu/libc.so.6

Opened: 1 times
Total duration: 0.11 ms
Avg session duration: 0.11 ms

Session 1 was open for 0.11ms (idle for 0.02ms)
- Event 1: Read 832 bytes

+---------+------+-------+
|         | Read | Write |
+========================+
| Total   | 832  | 0     |
|---------+------+-------|
| Average | 0    | 0     |
|---------+------+-------|
| Max     | 832  | 0     |
+---------+------+-------+

## File: /lib/x86_64-linux-gnu/libgcc_s.so.1

Opened: 1 times
Total duration: 0.09 ms
Avg session duration: 0.09 ms

Session 1 was open for 0.09ms (idle for 0.02ms)
- Event 1: Read 832 bytes

+---------+------+-------+
|         | Read | Write |
+========================+
| Total   | 832  | 0     |
|---------+------+-------|
| Average | 0    | 0     |
|---------+------+-------|
| Max     | 832  | 0     |
+---------+------+-------+

## File: /etc/passwd

Opened: 3 times
Total duration: 5000.43 ms
Avg session duration: 1666.81 ms

Session 1 was open for 2000.15ms (idle for 0.00ms)
Session 2 was open for 3000.21ms (idle for 3000.14ms)
- Event 1: Read 2183 bytes
Session 3 was open for 0.07ms (idle for 0.03ms)
- Event 1: Read 2183 bytes

+---------+------+-------+
|         | Read | Write |
+========================+
| Total   | 4366 | 0     |
|---------+------+-------|
| Average | 0    | 0     |
|---------+------+-------|
| Max     | 2183 | 0     |
+---------+------+-------+

## File: /etc/ld.so.cache

Opened: 1 times
Total duration: 0.04 ms
Avg session duration: 0.04 ms

Session 1 was open for 0.04ms (idle for 0.00ms)

+---------+------+-------+
|         | Read | Write |
+========================+
| Total   | 0    | 0     |
|---------+------+-------|
| Average | 0    | 0     |
|---------+------+-------|
| Max     | 0    | 0     |
+---------+------+-------+

## File: /proc/self/maps

Opened: 1 times
Total duration: 0.18 ms
Avg session duration: 0.18 ms

Session 1 was open for 0.18ms (idle for 0.09ms)
- Event 1: Read 1024 bytes
- Event 2: Read 1024 bytes
- Event 3: Read 1024 bytes

+---------+------+-------+
|         | Read | Write |
+========================+
| Total   | 3072 | 0     |
|---------+------+-------|
| Average | 0    | 0     |
|---------+------+-------|
| Max     | 1024 | 0     |
+---------+------+-------+
```
