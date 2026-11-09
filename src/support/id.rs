use std::time::{SystemTime, UNIX_EPOCH};

const HEX: &[u8] = b"0123456789abcdef";

pub fn uuid_v4() -> String {
    let mut raw = [0u8; 16];
    let mut seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|t| t.as_nanos() as u64)
        .unwrap_or(0x9e3779b97f4a7c15);
    let mut next = || {
        seed ^= seed << 13;
        seed ^= seed >> 7;
        seed ^= seed << 17;
        seed
    };
    for slot in raw.iter_mut() {
        *slot = (next() >> 24) as u8;
    }
    raw[6] = (raw[6] & 0x0f) | 0x40;
    raw[8] = (raw[8] & 0x3f) | 0x80;
    let mut out = String::with_capacity(36);
    for (index, byte) in raw.iter().enumerate() {
        if matches!(index, 4 | 6 | 8 | 10) {
            out.push('-');
        }
        out.push(HEX[(byte >> 4) as usize] as char);
        out.push(HEX[(byte & 0x0f) as usize] as char);
    }
    out
}
