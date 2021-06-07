const BITS_OF_BYTE: usize = 8;

/// clush message type
pub enum MessageType {
    Undefined,        // 0
    UserMessage,      // 1
    GroupMessage,     // 2
    UserFileMessage,  // 3
    GroupFileMessage, // 4
}

/// convert a given slice to u32
pub fn u32_from_bytes(bytes: &[u8]) -> Result<u32, &str> {
    if bytes.len() < 4 {
        return Err("insufficient data!");
    }
    let mut number: u32 = 0;
    for i in 0..4 {
        number <<= BITS_OF_BYTE;
        number |= bytes[i] as u32;
    }

    Ok(number)
}

/// convert a given slice to u64
pub fn u64_from_bytes(bytes: &[u8]) -> Result<u64, &str> {
    if bytes.len() < 8 {
        return Err("insufficient data!");
    }
    let mut number: u64 = 0;
    for i in 0..8 {
        number <<= BITS_OF_BYTE;
        number |= bytes[i] as u64;
    }

    Ok(number)
}

/// perform the conversion
pub fn u32_to_bytes(number: &u32) -> Vec<u8> {
    let mut bytes = vec![0u8; 0];
    let mut n = *number;
    for _ in 0..4 {
        bytes.insert(0, (n & 0xff) as u8);
        n >>= BITS_OF_BYTE;
    }

    bytes
}

/// perform the conversion
pub fn u64_to_bytes(number: &u64) -> Vec<u8> {
    let mut bytes = vec![0u8; 0];
    let mut n = *number;
    for _ in 0..8 {
        bytes.insert(0, (n & 0xff) as u8);
        n >>= BITS_OF_BYTE;
    }

    bytes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn u32_from_bytes_test() {
        let bytes = [0, 1, 2, 3, 4, 5, 6, 7, 8];
        assert_eq!(0x010203, u32_from_bytes(&bytes[..]).unwrap());
    }

    #[test]
    fn u64_from_bytes_test() {
        let bytes = [0, 1, 2, 3, 4, 5, 6, 7, 8];
        assert_eq!(0x01020304050607, u64_from_bytes(&bytes[..]).unwrap());
    }

    #[test]
    fn u32_to_bytes_test() {
        assert_eq!(vec![0, 1, 2, 3], u32_to_bytes(&0x010203));
    }

    #[test]
    fn u64_to_bytes_test() {
        assert_eq!(
            vec![0, 1, 2, 3, 4, 5, 6, 7],
            u64_to_bytes(&0x01020304050607)
        );
    }
}
