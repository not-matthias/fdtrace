fn main() {
    let content = std::fs::read_to_string("/etc/passwd").unwrap();
    let content = std::fs::read_to_string("/etc/passwd").unwrap();
    let content = std::fs::read_to_string("/etc/passwd").unwrap();
    let content = std::fs::read_to_string("/etc/passwd").unwrap();
    let content = std::fs::read_to_string("/etc/passwd").unwrap();
    let content = std::fs::read_to_string("/etc/passwd").unwrap();
    let content = std::fs::read_to_string("/etc/passwd").unwrap();
    let content = std::fs::read_to_string("/etc/passwd").unwrap();
    let content = std::fs::read_to_string("/etc/passwd").unwrap();
    let content = std::fs::read_to_string("/etc/passwd").unwrap();
    let content = std::fs::read_to_string("/etc/passwd").unwrap();
    let content = std::fs::read_to_string("/etc/passwd").unwrap();
}

// #![no_std]
// #![no_main]

// use core::panic::PanicInfo;

// // Constants for system calls
// const SYS_READ: usize = 0; // sys_read syscall number
// const SYS_OPEN: usize = 2; // sys_open syscall number
// const SYS_WRITE: usize = 1; // sys_write syscall number
// const SYS_EXIT: usize = 60; // sys_exit syscall number

// // File descriptor numbers
// const STDOUT: usize = 1; // Standard output
// const O_RDONLY: usize = 0; // Read-only flag for open

// fn syscall(number: usize, arg1: usize, arg2: usize, arg3: usize) -> usize {
//     let ret: usize;
//     unsafe {
//         core::arch::asm!(
//             "syscall",
//             inlateout("rax") number => ret,
//             inlateout("rdi") arg1 => _,
//             inlateout("rsi") arg2 => _,
//             inlateout("rdx") arg3 => _,
//             lateout("rcx") _, // clobbered register
//             lateout("r11") _, // clobbered register
//         );
//     }
//     ret
// }

// extern crate libc;

// #[no_mangle]
// pub extern "C" fn main(_argc: isize, _argv: *const *const u8) -> isize {
//     // Open /etc/passwd
//     let fd = unsafe { syscall(SYS_OPEN, "/etc/passwd\0".as_ptr() as usize,
// O_RDONLY, 0) };     let fd = unsafe { syscall(SYS_OPEN,
// "/etc/passwd\0".as_ptr() as usize, O_RDONLY, 0) };

//     if fd as isize <= 0 {
//         // Exit with error code 1 if open fails
//         unsafe { syscall(SYS_EXIT, 1, 0, 0) };
//     }

//     // Buffer to store file contents
//     let mut buffer = [0u8; 1024];
//     let bytes_read = unsafe { syscall(SYS_READ, fd, buffer.as_mut_ptr() as
// usize, buffer.len()) };

//     if bytes_read > 0 {
//         // Write contents to STDOUT
//         unsafe {
//             syscall(SYS_WRITE, STDOUT, buffer.as_ptr() as usize, bytes_read);
//         }
//     }

//     0
// }

// // Panic handler
// #[panic_handler]
// fn panic(_info: &PanicInfo) -> ! {
//     unsafe { syscall(SYS_EXIT, 1, 0, 0) };
//     loop {}
// }
