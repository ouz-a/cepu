use std::{
    io::{self, Read},
    os::unix::io::AsRawFd,
};

// macOS termios constants
const NCCS: usize = 20;
const VMIN: usize = 16;
const VTIME: usize = 17;
const ICANON: u64 = 0x100;
const ECHO: u64 = 0x008;
const ISIG: u64 = 0x080;
const TCSANOW: i32 = 0;

#[repr(C)]
#[derive(Clone, Copy)]
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
    fn tcgetattr(fd: i32, t: *mut Termios) -> i32;
    fn tcsetattr(fd: i32, action: i32, t: *const Termios) -> i32;
}

pub struct RawTerminal {
    original: Termios,
    fd: i32,
}

impl RawTerminal {
    pub fn new() -> io::Result<Self> {
        let fd = io::stdin().as_raw_fd();
        let mut original: Termios = unsafe { std::mem::zeroed() };

        if unsafe { tcgetattr(fd, &mut original) } != 0 {
            return Err(io::Error::last_os_error());
        }

        let mut raw = original;
        raw.c_lflag &= !(ICANON | ECHO | ISIG);
        // VMIN=0, VTIME=0: read returns immediately with whatever is available
        // (non-blocking)
        raw.c_cc[VMIN] = 0;
        raw.c_cc[VTIME] = 0;

        if unsafe { tcsetattr(fd, TCSANOW, &raw) } != 0 {
            return Err(io::Error::last_os_error());
        }

        Ok(RawTerminal { original, fd })
    }

    pub fn try_read(&self) -> Option<u8> {
        let mut buf = [0u8; 1];
        match io::stdin().read(&mut buf) {
            Ok(1) => Some(buf[0]),
            _ => None,
        }
    }
}

impl Drop for RawTerminal {
    fn drop(&mut self) {
        unsafe { tcsetattr(self.fd, TCSANOW, &self.original) };
    }
}
