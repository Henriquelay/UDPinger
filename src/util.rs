pub fn u8_to_py_hex_str(bytes: &[u8]) -> String {
    let mut hex_str = String::new();
    for byte in bytes {
        hex_str.push_str(&format!("\\x{:02x}", byte));
    }
    hex_str
}
