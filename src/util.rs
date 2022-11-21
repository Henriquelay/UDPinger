pub fn u8_to_py_hex_str(bytes: &[u8]) -> String {
    let mut hex_str = String::new();
    for byte in bytes {
        hex_str.push_str(&format!("\\x{:02x}", byte));
    }
    hex_str
}

pub fn seq_to_u64(seq: &[u8]) -> u64 {
    u64::from_le_bytes([seq[0], seq[1], seq[2], seq[3], seq[4], 0, 0, 0])
}
