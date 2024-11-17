#![no_std]

#[repr(C)]
pub struct SyscallLog {
    pub pid: u32,
    pub id: u32,
    pub ts: u64,
    pub args: (),
    pub cmd: [u8; 16],
}

impl SyscallLog {
    pub fn cmd(&self) -> &str {
        unsafe { core::str::from_utf8_unchecked(&self.cmd[..]) }
    }

    #[inline(always)]
    pub fn from_bytes(bytes: &[u8]) -> Option<SyscallLog> {
        Some(unsafe { (bytes.as_ptr() as *const SyscallLog).read_unaligned() })
    }
}

impl core::fmt::Debug for SyscallLog {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("SyscallLog")
            .field("pid", &self.pid)
            .field("id", &self.id)
            .field("ts", &self.ts)
            .field("cmd", &self.cmd())
            .finish()
    }
}
