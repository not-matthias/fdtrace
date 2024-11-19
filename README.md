# fdtrace

# TODO

sudo -E cargo rr '/run/current-system/sw/bin/grep needle /home/not-matthias/Documents' --debug

## Prerequisites

- Linux Kernel > TODO
- [Rust](https://www.rust-lang.org/tools/install)
- [bpftrace](TODO): `sudo apt install -y bpftrace`

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
