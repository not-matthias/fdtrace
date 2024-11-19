# fdtrace

## Prerequisites

- Linux Kernel > TODO
- [Rust](https://www.rust-lang.org/tools/install)
- [bpftrace](TODO): `sudo apt install -y bpftrace`

## Run

```
> cd fdtrace
> sudo -E cargo rr --help
> sudo -E cargo rr --debug (whereis ls)
```

To run the example:
```
> cargo br
> sudo -E cargo rr target/release/example
```


## Debugging

Run your command with `strace` and compare the syscalls:
```
strace ls 2> strace.txt
```

Run fdtrace with `--debug` which creates `debug.txt`:
```
sudo -E cargo rr /run/current-system/sw/bin/ls --debug
```

Compare `debug.txt` and `strace.txt`.
