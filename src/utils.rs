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

#[inline]
pub const fn align(value: u64, align: u64) -> u64 {
    debug_assert!(align.is_power_of_two());
    value & !(align - 1)
}

#[inline]
pub const fn is_aligned(value: u64, align: u64) -> bool {
    return (value & (align - 1)) == 0;
}
