use std::os::fd::AsRawFd;

const NCCS: usize = 20;
const TCSANOW: i32 = 0;
const F_SETFL: i32 = 4;
const F_GETFL: i32 = 3;
const O_NONBLOCK: i32 = 0x0004;
const ICANON: u64 = 0x00000100;
const ECHO: u64 = 0x00000008;

#[repr(C)]
#[derive(Clone)]
struct Termios {
    c_iflag: u64,
    c_oflag: u64,
    c_cflag: u64,
    c_lflag: u64,
    c_cc: [u8; NCCS],
    c_ispeed: u64,
    c_ospeed: u64,
}

unsafe extern "C" {
    fn tcgetattr(fd: i32, termios: *mut Termios) -> i32;
    fn tcsetattr(fd: i32, action: i32, termios: *const Termios) -> i32;
    fn fcntl(fd: i32, cmd: i32, ...) -> i32;
    fn read(fd: i32, buf: *mut u8, count: usize) -> isize;
}

pub struct RawTerminal {
    old: Termios,
}

impl RawTerminal {
    pub fn enable() -> Self {
        let fd = std::io::stdin().as_raw_fd();
        let mut old = unsafe { std::mem::zeroed::<Termios>() };
        unsafe { tcgetattr(fd, &mut old) };

        let mut raw = old.clone();
        raw.c_lflag &= !(ICANON | ECHO);
        unsafe { tcsetattr(fd, TCSANOW, &raw) };

        let flags = unsafe { fcntl(fd, F_GETFL) };
        unsafe { fcntl(fd, F_SETFL, flags | O_NONBLOCK) };

        RawTerminal { old }
    }
}

impl Drop for RawTerminal {
    fn drop(&mut self) {
        let fd = std::io::stdin().as_raw_fd();
        unsafe { tcsetattr(fd, TCSANOW, &self.old) };
        let flags = unsafe { fcntl(fd, F_GETFL) };
        unsafe { fcntl(fd, F_SETFL, flags & !O_NONBLOCK) };
    }
}

/// Try to read one byte from stdin. Returns None if nothing available.
pub fn try_read_byte() -> Option<u8> {
    let mut byte = 0u8;
    let n = unsafe { read(0, &mut byte, 1) };
    if n == 1 { Some(byte) } else { None }
}
