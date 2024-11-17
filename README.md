# fdtrace

# TODO

- [ ] Setup github actions CI/CD

## Prerequisites

1. Nightly Rust toolchain
  - `rustup toolchain install nightly`
  - `rustup component add rust-src`
2. LLVM TODO
3. `cargo install bpf-linker --no-default-features`


## Errors


### Failed to create map

```
Finished `release` profile [optimized] target(s) in 16.38s
 Running `/home/not-matthias/Documents/technical/git/syscall-tracer/target/release/fdtrace`
[WARN  fdtrace] cannot remove mem lock
Error: map error: failed to create map `AYA_LOG_BUF` with code -1

Caused by:
0: failed to create map `AYA_LOG_BUF` with code -1
1: Operation not permitted (os error 1)
```


Run with sudo:
```
[16:22] not-matthias:fdtrace (main)> sudo RUST_LOG=debug ../target/release/fdtrace
Waiting for Ctrl-C...
[INFO  fdtrace] received a packet
[INFO  fdtrace] received a packet
[INFO  fdtrace] received a packet
[INFO  fdtrace] received a packet
[INFO  fdtrace] received a packet
[INFO  fdtrace] received a packet
[INFO  fdtrace] received a packet
[INFO  fdtrace] received a packet
[INFO  fdtrace] received a packet
[INFO  fdtrace] received a packet
[INFO  fdtrace] received a packet
[INFO  fdtrace] received a packet
^CExiting...
[16:22] not-matthias:fdtrace (main)>
```
