pub const fn mb(n: usize) -> usize {
    n << 20
}

pub const MAP_PRIVATE: i32 = 0x0002;
pub const MAP_ANON: i32 = 0x1000;
pub const PROT_READ: i32 = 0x01;
pub const PROT_WRITE: i32 = 0x02;
pub const MEMORY_SIZE: usize = mb(256);

unsafe extern "C" {
    pub fn mmap(addr: *mut u8, len: usize, prot: i32, flags: i32, fd: i32, offset: i64) -> *mut u8;
}
