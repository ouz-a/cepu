use std::ops::Not;

use crate::instruction::UNDEF_PANIC;

#[macro_export]
macro_rules! get_bits_ct {
    ($val:expr, $start:expr, $len:expr) => {{
        const START: usize = $start;
        const LEN: usize = $len;
        (($val >> START) & ((1 << LEN) - 1))
    }};
}

/// I find these bit operations always confusing
///
/// Example: let b = `0b1011_1111_1111_1110_...`
///
/// If you wanted to get top `16`s bits
/// You need to `64` - `48`
///
/// Thus:
/// start: `48`
/// len: `16`
///
/// you would get: `0b1011_1111_1111_1110`
#[inline(always)]
pub fn bits_get(val: u64, start: u8, len: u8) -> u64 {
    debug_assert!((start as u16 + len as u16) <= 64);

    if len == 0 {
        return 0;
    }

    let mask = if len == 64 { u64::MAX } else { (1u64 << len) - 1 };

    (val >> start) & mask
}

pub fn bits_get_u128(val: u128, start: u8, len: u8) -> u128 {
    debug_assert!((start as u16 + len as u16) <= 128);

    if len == 0 {
        return 0;
    }

    let mask = if len == 128 { u128::MAX } else { (1u128 << len) - 1 };

    (val >> start) & mask
}

/// Gets bits then shifts them to left by `start` amount
/// This has the same effect as masking value
pub fn bits_get_in_place(val: u64, start: u8, len: u8) -> u64 {
    bits_get(val, start, len) << start
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
    (val ^ sign_bit).wrapping_sub(sign_bit)
}

/// We preserve SIGN of a value while extending it
/// Please enter width of the value, not width you are extenting into
pub fn sign_extend(val: u64, width: u8) -> u64 {
    if width == 64 {
        return val;
    }
    let shift = 64 - width;
    ((val << shift) as i64 >> shift) as u64
}

pub fn zero_extend(val: u64, width: u8) -> u64 {
    let mask = if width >= 63 { u64::MAX } else { u64::MAX >> (63 - width as u32) };
    val & mask
}

pub fn insert_16bit_field(mut target: u64, value: u16, start_bit: u32) -> u64 {
    let sixteen_bit_mask = 0xFFFF_u64 << start_bit;

    target &= !sixteen_bit_mask;

    let value_at_position = (value as u64) << start_bit;

    target |= value_at_position;

    target
}

/// This function decodes compressed bit pattern encodings into full witdh
/// bitmasks.
///
/// # Arguments
/// * `immn` - Used to decode length of the element size.
/// * `imms` - Encodes the number of consecutive set bits (ones) in the pattern.
/// * `immr` - The number of positions to rotate right.
/// * `immediate` - Flag indicating whether this is for a logical immediate
///   instruction.
/// * `m` - The total width of the output masks in bits (32 or 64).
///
/// # Returns
/// (`wmask`,`tmask`)
///
/// `wmask` - Rotated tun of ones replicated across M.
///
/// `tmask` - Trailing mask, a contiguous from LSB mask replicated across M.
pub fn decode_bit_mask(immn: bool, imms: u8, immr: u8, immediate: bool, m: u8) -> (u64, u64) {
    let mask: u8 = (1 << 6) - 1;
    let imms_not = (imms.not()) & mask;

    let concatenated = if immn { (1 << 6) | (imms_not as u64) } else { imms_not as u64 };

    // Find the highest set bit position in the 7-bit value
    // log2 of element size
    let len = concatenated.highest_one().unwrap_or_default();

    if len == 0 {
        panic!("Undefined");
    }
    assert!(m >= (1 << len));
    // Ensures we only use bits appropriate for the current element size
    let levels = zero_extend((1 << len) - 1, 6);
    if immediate && ((imms as u64 & levels) == levels) {
        panic!("Undefined");
    }

    let s = imms as u64 & levels;
    let r = immr as u64 & levels;

    let diff = (s.wrapping_sub(r)) & 0x3F;

    let esize = 1 << len;
    let d = bits_get(diff, 0, (len).try_into().unwrap());
    let welem = ones_masked((s + 1) as u32, esize);
    let telem = ones_masked((d + 1) as u32, esize);
    let wmask = replicate_bits_u64(
        rot_right_width(welem, esize.into(), r.try_into().unwrap()),
        esize.into(),
        (m / esize).into(),
    );
    let tmask = replicate_bits_u64(telem, esize.into(), (m / esize).into());
    (wmask, tmask)
}

/// Rotates bits to the right within a specified bit width.
///
/// This exists because in Rust without constraining the bit width.
///
/// One could not express rotation within certain bit range.
///
/// # Arguments
/// * `bits` - The value whose bits will be rotated
/// * `width` - The bit width to constrain the rotation (1-64)
/// * `amount` - The number of positions to rotate right
///
/// # Returns
/// The rotated value, masked to the specified width
///
/// # Panics
/// Panics if `width` is 0 or greater than 64
pub fn rot_right_width(bits: u64, width: u32, amount: u32) -> u64 {
    assert!(width > 0 && width <= 64);
    let amount = amount % width;
    let mask = if width == 64 { u64::MAX } else { (1u64 << width) - 1 };
    let x = bits & mask;
    let left = if amount == 0 { 0 } else { x << (width - amount) };
    ((x >> amount) | left) & mask
}

pub fn replicate_bits_u64(bits: u64, width: u32, times: u32) -> u64 {
    assert!(width <= 64 && times <= 64 && (width as u64) * (times as u64) <= 64);
    if width == 0 || times == 0 {
        return 0;
    }

    let mask_m = if width == 64 { u64::MAX } else { (1u64 << width) - 1 };
    let p = bits & mask_m;

    let mn = width * times;
    let numer = if mn == 64 { u64::MAX } else { (1u64 << mn) - 1 };

    let factor = if width == 64 { 1 } else { numer / mask_m };
    p.wrapping_mul(factor)
}

fn replicate_bits_128(bits: u128, width: u8, times: u8) -> u128 {
    assert!(width <= 128 && times <= 128 && (width as u128) * (times as u128) <= 128);
    if width == 0 || times == 0 {
        return 0;
    }

    let mask_m = if width == 128 { u128::MAX } else { (1u128 << width) - 1 };
    let p = bits & mask_m;

    let mn = width * times;
    let numer = if mn == 128 { u128::MAX } else { (1u128 << mn) - 1 };

    let factor = if width == 128 { 1 } else { numer / mask_m };
    p.wrapping_mul(factor)
}

fn ones_masked(n: u32, width: u8) -> u64 {
    let ones = if n == 0 {
        0
    } else if n >= 64 {
        u64::MAX
    } else {
        (1u64 << n) - 1
    };
    zero_extend(ones, width)
}

// Read element
pub fn elem_get(vec: u128, e: usize, esize: usize) -> u64 {
    let shift = e * esize;
    let mask = (1u128 << esize) - 1;
    ((vec >> shift) & mask) as u64
}

// Write element
pub fn elem_set(vec: u128, e: usize, esize: usize, val: u64) -> u128 {
    let shift = e * esize;
    let mask = (1u128 << esize) - 1;
    (vec & !(mask << shift)) | ((val as u128 & mask) << shift)
}

pub trait Utils {
    fn datasize_sf(self) -> u8;
}

impl Utils for bool {
    fn datasize_sf(self) -> u8 {
        if self { 64 } else { 32 }
    }
}

pub fn just_panic() {
    UNDEF_PANIC.store(true, std::sync::atomic::Ordering::SeqCst);
}

pub trait AsU128: Sized {
    fn to_u128(self) -> u128;
    fn from_u128(val: u128) -> Self;
}

macro_rules! impl_as_u128 {
    ($($t:ty),*) => {
        $(
            impl AsU128 for $t {
                fn to_u128(self) -> u128 { self as u128 }
                fn from_u128(val: u128) -> Self { val as $t }
            }
        )*
    };
}

impl_as_u128!(u8, u16, u32, u64, u128);

pub trait BitUtils {
    fn single_bit(self, idx: u8) -> u8;
    fn bits_get(self, start: u8, len: u8) -> Self;
    /// Set a single bit to 1
    fn bit_set(self, idx: u8) -> Self;
    /// This should be only used for unsigned types
    fn replicate<T: AsU128>(self, repeat: u8) -> T;
}

macro_rules! impl_bit_utils {
    ($($t:ty),*) => {
        $(
            impl BitUtils for $t {
                fn single_bit(self, idx: u8) -> u8 {
                    bits_get_u128(self as u128, idx, 1) as u8
                }

                fn bits_get(self, start: u8, len: u8) -> Self {
                    bits_get_u128(self as u128, start, len) as Self
                }

                fn bit_set(self, idx: u8) -> Self {
                    self | (1 << idx)
                }

                fn replicate<T:AsU128>(self,repeat:u8) -> T {
                    let size_of = size_of::<T>() as u8;
                    let res = replicate_bits_128(self as u128,size_of,repeat);
                    T::from_u128(res)
                }
            }
        )*
    };
}

impl_bit_utils!(u8, u16, u32, u64, u128, usize);

macro_rules! impl_bit_utils_signed {
    ($(($signed:ty, $unsigned:ty)),*) => {
        $(
            impl BitUtils for $signed {
                fn single_bit(self, idx: u8) -> u8 {
                    bits_get_u128(self as $unsigned as u128, idx, 1) as u8
                }

                fn bits_get(self, start: u8, len: u8) -> Self {
                    bits_get_u128(self as $unsigned as u128, start, len) as Self
                }

                fn bit_set(self, idx: u8) -> Self {
                    self | (1 << idx)
                }

               fn replicate<T:AsU128>(self,repeat:u8) -> T {
                    let size_of = size_of::<T>() as u8;
                    let res = replicate_bits_128(self as u128,size_of,repeat);
                    T::from_u128(res)
                }
            }
        )*
    };
}

impl_bit_utils_signed!((i8, u8), (i16, u16), (i32, u32), (i64, u64), (i128, u128), (isize, usize));
