//! Generic reusable types.

use std::fmt;
use std::str::FromStr;

pub const MSB_MASK: u8 = 0b1000_0000;
const VARINT_PAYLOAD_MASK: u8 = 0b0111_1111;
const VARINT_PAYLOAD_BITS: u8 = 7;

/// Semantic version.
///
/// Extensions are not (yet) supported, i.e., `1.0.0-alpha+001`.
///
/// 16-bit should suffice for each part. If not, you might want to look at your versioning.
///
/// ## Examples
///
/// It can be created in different ways.
///
/// ```
/// use ers::SemVer;
/// use std::str::FromStr;
/// assert_eq!(SemVer::new(), SemVer{ major: 0, minor: 0, patch: 0 });
///
/// let expected = Ok(SemVer{ major: 1, minor: 23, patch: 456 });
/// assert_eq!(SemVer::from_str("1.23.456"), expected);
/// assert_eq!("1.23.456".parse(), expected);
///
/// let input = String::from("1.23.456");
/// assert_eq!(input.parse(), expected);
/// assert_eq!(input.parse::<SemVer>(), expected);
/// ```
///
/// Incomplete versions are not valid.
///
/// ```
/// use ers::SemVer;
/// use std::str::FromStr;
/// assert!(SemVer::from_str("1.23").is_err());
/// ```
///
/// ## Reference
///
/// <https://semver.org>
///
#[derive(Debug, Eq, PartialEq, PartialOrd, Ord, Hash)]
pub struct SemVer {
    pub major: u16,
    pub minor: u16,
    pub patch: u16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SemVerErrorKind {
    /// The given string cannot be decomposed into major-minor-patch parts.
    InvalidStructure,

    // TODO(feat): other types of errors
}

#[derive(Debug, Eq, PartialEq)]
pub struct ParseSemVerError {
    pub kind: SemVerErrorKind,
}

impl fmt::Display for ParseSemVerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.kind {
            SemVerErrorKind::InvalidStructure => "Could not parse `SemVer`",
        }.fmt(f)
    }
}

impl FromStr for SemVer {
    type Err = ParseSemVerError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use self::SemVerErrorKind::*;

        let xs = s.split(".")
            .filter_map(|x| x.parse::<u16>().ok())
            .collect::<Vec<u16>>();

        let [major, minor, patch] = xs[..]
            else { return Err(ParseSemVerError { kind: InvalidStructure }); };
        Ok(Self {
            major,
            minor,
            patch,
        })
    }
}

impl SemVer {

    /// Creates a default initialized `SemVer`.
    ///
    /// ## Example
    ///
    /// ```
    /// use ers::SemVer;
    /// let version = SemVer::new();
    /// # assert_eq!(version, SemVer{ major: 0, minor: 0, patch: 0});
    /// ```
    pub fn new() -> SemVer {
        Self {
            major: 0,
            minor: 0,
            patch: 0
        }
    }
}

/// Variable-length integer.
///
/// MSB (most significant bit) in each byte is the continuation bit indicating
/// whether the following byte is a part of the `varint`.
///
/// The concatenated bytes (without continuation bits) is in
/// little-endian.
///
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Varint {
    value: i128,
    length: u8
}

impl TryFrom<&[u8]> for Varint {
    type Error = &'static str;

    /// Construct a [`Varint`] from bytes.
    ///
    /// ## Examples
    ///
    /// ```
    /// use ers::Varint;
    /// let empty = Varint::try_from(vec![].as_slice());
    /// assert_eq!(empty, Err("Insufficient data"));
    /// ```
    ///
    /// ```
    /// use ers::Varint;
    /// let x = Varint::try_from(vec![0b0100_0001].as_slice()).unwrap();
    /// assert_eq!(x.value(), 65);
    /// assert_eq!(x.length(), 1);
    /// ```
    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        let mut value: i128 = 0;
        let mut length: u8 = 0;

        let mut continuation_bit = true;

        while continuation_bit {
            if length as usize >= data.len() {
                return Err("Insufficient data");
            }

            let byte = data[length as usize];
            continuation_bit = byte & MSB_MASK != 0;
            value += ((byte & VARINT_PAYLOAD_MASK) << length * VARINT_PAYLOAD_BITS) as i128;

            length += 1;
        }

        Ok(Self{ value, length })
    }
}

// TODO: other integer types (macro maybe?)
impl From<Varint> for i128 {
    fn from(value: Varint) -> Self {
        value.value()
    }
}

// TODO(feat): encode into varint as bytes

impl Varint {
    pub fn value(&self) -> i128 {
        self.value
    }

    pub fn length(&self) -> u8 {
        self.length
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_invalid_semver_construct() {
        assert!(SemVer::from_str("2.23").is_err());
        assert!(SemVer::from_str("2.23.456.7").is_err());
        assert!(SemVer::from_str("2.23.A").is_err());
    }

    #[test]
    fn test_semver_error_format() {
        let error_msg = SemVer::from_str("2.23")
            .err()
            .unwrap()
            .to_string();

        assert_eq!(error_msg, "Could not parse `SemVer`".to_string());
    }

    #[test]
    fn test_varint_decode_incomplete() {
        assert!(Varint::try_from(vec![0b1001_0100].as_slice()).is_err());
    }

    #[test]
    fn test_varint_decode_2_bytes() {
        let x = Varint::try_from(vec![0b1001_0110, 0b0000_0001].as_slice()).unwrap();
        assert_eq!(x.value(), 150);
        assert_eq!(x.length(), 2);
    }

    #[test]
    fn test_varint_decode_conversion_into() {
        let varint: Varint = vec![0b1001_0110, 0b0000_0001].as_slice().try_into().unwrap();
        let num: i128 = varint.into();
        assert_eq!(num, 150);
    }
}
