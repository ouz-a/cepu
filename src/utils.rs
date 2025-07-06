#[macro_export]
macro_rules! get_bits_ct {
    ($val:expr, $start:expr, $len:expr) => {{
        const START: usize = $start;
        const LEN: usize = $len;
        (($val >> START) & ((1 << LEN) - 1))
    }};
}

#[inline]
pub const fn align(value: u64, align: u64) -> u64 {
    debug_assert!(align.is_power_of_two());
    value & !(align - 1)
}

#[inline]
pub const fn is_aligned(value: u64, align: u64) -> bool {
    (value & (align - 1)) == 0
}

pub fn sign_extend_xor(val: u64, width: isize) -> u64 {
    let sign_bit = 1u64 << (width - 1);
    (val ^ sign_bit) - sign_bit
}
