#[macro_export]
macro_rules! get_bits_ct {
    ($val:expr, $start:expr, $len:expr) => {{
        const START: usize = $start;
        const LEN: usize = $len;
        (($val >> START) & ((1 << LEN) - 1))
    }};
}

pub const fn get_bits_u32(val: u32, start: usize, len: usize) -> u32 {
    (val >> start) & ((1u32 << len) - 1)
}
