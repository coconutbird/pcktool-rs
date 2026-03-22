//! Wwise hash utilities.

/// Computes the FNV1-32 hash used by Wwise for asset name → ID mapping.
///
/// The input is lowercased before hashing. File extensions are stripped.
pub fn fnv1_32(input: &str) -> u32 {
    if input.is_empty() {
        return 0;
    }

    // Strip file extension if present
    let name = match input.rfind('.') {
        Some(pos) if pos > 0 => &input[..pos],
        _ => input,
    };

    let mut hash: u32 = 0x811C_9DC5;
    for b in name.bytes() {
        let b = b.to_ascii_lowercase();
        hash = hash.wrapping_mul(0x0100_0193);
        hash ^= b as u32;
    }
    hash
}

/// Computes the Wwise FOURCC tag from four ASCII characters (little-endian packed).
///
/// ```
/// # use pcktool::hash::fourcc;
/// assert_eq!(fourcc(b"AKPK"), 0x4B504B41);
/// ```
pub const fn fourcc(chars: &[u8; 4]) -> u32 {
    (chars[0] as u32)
        | ((chars[1] as u32) << 8)
        | ((chars[2] as u32) << 16)
        | ((chars[3] as u32) << 24)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fourcc_akpk() {
        assert_eq!(fourcc(b"AKPK"), 0x4B50_4B41);
    }

    #[test]
    fn test_fourcc_bkhd() {
        assert_eq!(fourcc(b"BKHD"), 0x4448_4B42);
    }

    #[test]
    fn test_fnv1_empty() {
        assert_eq!(fnv1_32(""), 0);
    }

    #[test]
    fn test_fnv1_strips_extension() {
        let with_ext = fnv1_32("test.bnk");
        let without_ext = fnv1_32("test");
        assert_eq!(with_ext, without_ext);
    }

    #[test]
    fn test_fnv1_case_insensitive() {
        assert_eq!(fnv1_32("Hello"), fnv1_32("hello"));
    }
}
