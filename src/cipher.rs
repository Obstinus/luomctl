//! Cipher for the 8-byte feature reports.
//! Recovered from `cmd_encrypt` (0x42B3C0) and `cmd_decrypt` (0x42B580) in Combaterwing GM.exe.

const KEY: [u8; 8] = [0, b'4', b'5', b'2', b'A', b'E', b'A', 0];
const TABLE: &[u8; 8] = b"RoNgtEng";

fn add(i: usize) -> u8 {
    TABLE[i].rotate_left(4)
}

pub fn encrypt(p: [u8; 8]) -> [u8; 8] {
    let mut r = [p[5], p[4], p[7], p[6], p[1], p[0], p[3], p[2]];
    for i in 1..7 {
        r[i] ^= KEY[i];
    }
    let mut r = u64::from_be_bytes(r).rotate_left(3).to_be_bytes();
    for (i, b) in r.iter_mut().enumerate() {
        *b = b.wrapping_add(add(i));
    }
    r
}

pub fn decrypt(c: [u8; 8]) -> [u8; 8] {
    let mut r = c;
    for (i, b) in r.iter_mut().enumerate() {
        *b = b.wrapping_sub(add(i));
    }
    let mut r = u64::from_be_bytes(r).rotate_right(3).to_be_bytes();
    for i in 1..7 {
        r[i] ^= KEY[i];
    }
    [r[5], r[4], r[7], r[6], r[1], r[0], r[3], r[2]]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hex(s: &str) -> [u8; 8] {
        let v: Vec<u8> = (0..16).step_by(2).map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap()).collect();
        v.try_into().unwrap()
    }

    // Pairs taken from a USB capture of the Windows tool.
    #[test]
    fn matches_capture() {
        for (plain, wire) in [
            ("00aadd3b620000db", "27ad550da17fbc5e"),
            ("12014000000000ac", "269aad08490ef076"),
            ("0d020502000000e9", "269cc5086196fe9e"),
        ] {
            assert_eq!(encrypt(hex(plain)), hex(wire));
            assert_eq!(decrypt(hex(wire)), hex(plain));
        }
    }
}
