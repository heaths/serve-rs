// Copyright 2025 Heath Stewart.
// Licensed under the MIT License. See LICENSE.txt in the project root for license information.

use std::{fmt, ops::Deref, str::FromStr};

/// Represents size.
///
/// This encapsulates a `usize`, which you can get by dereferencing this `Size`.
///
/// # Examples
///
/// ```
/// use serve::Size;
/// use std::str::FromStr as _;
///
/// # fn main() -> Result<(), serve::ParseSizeError> {
/// let size: Size = "1 MB".parse()?;
/// assert_eq!(format!("{size:#}"), "976.56 KiB");
/// # Ok(())
/// # }
/// ```
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Ord, PartialOrd)]
pub struct Size(usize);

impl Deref for Size {
    type Target = usize;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl fmt::Display for Size {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let suffix = if f.alternate() {
            match self.0 {
                d if d >= Suffix::PiB as u64 as usize => Suffix::PiB,
                d if d >= Suffix::TiB as u64 as usize => Suffix::TiB,
                d if d >= Suffix::GiB as usize => Suffix::GiB,
                d if d >= Suffix::MiB as usize => Suffix::MiB,
                d if d >= Suffix::KiB as usize => Suffix::KiB,
                _ => Suffix::B,
            }
        } else {
            match self.0 {
                d if d >= Suffix::PB as u64 as usize => Suffix::PB,
                d if d >= Suffix::TB as u64 as usize => Suffix::TB,
                d if d >= Suffix::GB as usize => Suffix::GB,
                d if d >= Suffix::MB as usize => Suffix::MB,
                d if d >= Suffix::KB as usize => Suffix::KB,
                _ => Suffix::B,
            }
        };
        let val = self.0 as f64 / suffix as u64 as f64;
        // cspell:ignore fract
        let p = if val.fract() == 0f64 { 0 } else { 2 };
        write!(f, "{val:.*} {}", p, suffix)
    }
}

impl From<usize> for Size {
    fn from(value: usize) -> Self {
        Self(value)
    }
}

macro_rules! impl_from {
    ($($t:ty)+) => { $(
        impl From<$t> for Size {
            fn from(value: $t) -> Self {
                Self(value as usize)
            }
        }
    )* }
}

impl_from!(u8 u16 u32 u64 u128);
impl_from!(i8 i16 i32 i64 i128 isize);

impl FromStr for Size {
    type Err = ParseSizeError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        const ZERO: u32 = '0' as u32;

        if s.is_empty() {
            return Err(SizeErrorKind::Empty.into());
        }

        let mut size: usize = 0;
        let mut last: Option<u8> = None;

        for (i, ch) in s.bytes().enumerate() {
            match ch {
                b'-' => return Err(SizeErrorKind::NegOverflow.into()),
                b'0'..=b'9' => {
                    size *= 10;
                    size += (ch as u32).wrapping_sub(ZERO) as usize;
                }
                b',' | b'_' | b' ' if matches!(last, Some(c) if c.is_ascii_digit()) => {}
                _ if i > 0 => {
                    let suffix: Suffix = s[i..].parse()?;
                    size *= suffix as usize;
                    break;
                }
                _ => return Err(SizeErrorKind::InvalidDigit.into()),
            }
            last = Some(ch);
        }

        Ok(Size(size))
    }
}

/// Error returned by [`Size::from_str()`].
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParseSizeError {
    pub(crate) kind: SizeErrorKind,
}

/// Error kind for [`ParseSizeError`].
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SizeErrorKind {
    /// The string to parse was empty.
    Empty,

    /// Parsed a character that was not a digit.
    InvalidDigit,

    /// The suffix was invalid.
    ///
    /// Case-insensitive suffixes include: b, kb, kib, mb, mib, gb, gib, tb, tib, pb, and pib.
    InvalidSuffix,

    /// The number was negative.
    NegOverflow,
}

impl From<SizeErrorKind> for ParseSizeError {
    fn from(value: SizeErrorKind) -> Self {
        Self { kind: value }
    }
}

impl fmt::Display for ParseSizeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.kind {
            SizeErrorKind::Empty => f.write_str("empty"),
            SizeErrorKind::InvalidDigit => f.write_str("invalid digit"),
            SizeErrorKind::InvalidSuffix => f.write_str("invalid suffix"),
            SizeErrorKind::NegOverflow => f.write_str("negative overflow"),
        }
    }
}

impl std::error::Error for ParseSizeError {}

#[derive(Copy, Clone, Debug)]
#[repr(u64)]
enum Suffix {
    B = 1,
    KB = 1_000,
    KiB = 1 << 10,
    MB = 1_000_000,
    MiB = 1 << 20,
    GB = 1_000_000_000,
    GiB = 1 << 30,
    TB = 1_000_000_000_000,
    TiB = 1 << 40,
    PB = 1_000_000_000_000_000,
    PiB = 1 << 50,
}

impl fmt::Display for Suffix {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Suffix::B => "B",
            Suffix::KB => "KB",
            Suffix::KiB => "KiB",
            Suffix::MB => "MB",
            Suffix::MiB => "MiB",
            Suffix::GB => "GB",
            Suffix::GiB => "GiB",
            Suffix::TB => "TB",
            Suffix::TiB => "TiB",
            Suffix::PB => "PB",
            Suffix::PiB => "PiB",
        };
        f.write_str(s)
    }
}

impl FromStr for Suffix {
    type Err = ParseSizeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "b" => Ok(Suffix::B),
            "kb" => Ok(Suffix::KB),
            "kib" => Ok(Suffix::KiB),
            "mb" => Ok(Suffix::MB),
            "mib" => Ok(Suffix::MiB),
            "gb" => Ok(Suffix::GB),
            "gib" => Ok(Suffix::GiB),
            "tb" => Ok(Suffix::TB),
            "tib" => Ok(Suffix::TiB),
            "pb" => Ok(Suffix::PB),
            "pib" => Ok(Suffix::PiB),
            _ => Err(SizeErrorKind::InvalidSuffix.into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn size_parse() {
        let size: Size = "1mb".parse().unwrap();
        assert_eq!(*size, 1000 * 1000);

        let size: Size = "1 MiB".parse().unwrap();
        assert_eq!(*size, 1024 * 1024);
    }

    #[test]
    fn size_parse_empty() {
        assert!(matches!(
            "".parse::<Size>(),
            Err(err) if err.kind == SizeErrorKind::Empty,
        ));
    }

    #[test]
    fn size_parse_invalid_digit() {
        assert!(matches!(
            "pib".parse::<Size>(),
            Err(err) if err.kind == SizeErrorKind::InvalidDigit,
        ));

        assert!(matches!(
            "b4".parse::<Size>(),
            Err(err) if err.kind == SizeErrorKind::InvalidDigit,
        ));
    }

    #[test]
    fn size_parse_invalid_suffix() {
        assert!(matches!(
            "1 nibble".parse::<Size>(),
            Err(err) if err.kind == SizeErrorKind::InvalidSuffix,
        ));
    }

    #[test]
    fn size_parse_neg_overflow() {
        assert!(matches!(
            "-1kb".parse::<Size>(),
            Err(err) if err.kind == SizeErrorKind::NegOverflow,
        ));
    }

    #[test]
    fn size_fmt() {
        let size: Size = "1 gb".parse().unwrap();
        assert_eq!(format!("{size}"), "1 GB");
        assert_eq!(format!("{size:#}"), "953.67 MiB");

        let size: Size = "1 gib".parse().unwrap();
        assert_eq!(format!("{size}"), "1.07 GB");
        assert_eq!(format!("{size:#}"), "1 GiB");

        let size: Size = "1b".parse().unwrap();
        assert_eq!(format!("{size}"), "1 B");
        assert_eq!(format!("{size:#}"), "1 B");
    }
}
