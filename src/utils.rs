#[macro_export]
macro_rules! get_bits_ct {
    ($val:expr, $start:expr, $len:expr) => {{
        const START: usize = $start;
        const LEN: usize = $len;
        (($val >> START) & ((1 << LEN) - 1))
    }};
}
