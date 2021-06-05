const BITS_OF_BYTE: usize = 8;

pub enum MessageType {
    None,
}

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
}
