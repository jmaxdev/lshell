use std::env;

fn get_machine_key() -> Vec<u8> {
    let username = env::var("USERNAME")
        .or_else(|_| env::var("USER"))
        .unwrap_or_else(|_| "lshell_user".to_string());
    let computer = env::var("COMPUTERNAME")
        .or_else(|_| env::var("HOSTNAME"))
        .unwrap_or_else(|_| "lshell_host".to_string());

    let raw_key = format!("lshell_secret_v1_{}_{}", username, computer);

    let mut key = vec![0u8; 32];
    for (i, byte) in raw_key.bytes().enumerate() {
        key[i % 32] ^= byte.wrapping_add((i * 31 + 17) as u8);
    }
    key
}

fn hex_encode(data: &[u8]) -> String {
    data.iter().map(|b| format!("{:02x}", b)).collect()
}

fn hex_decode(s: &str) -> Option<Vec<u8>> {
    if !s.len().is_multiple_of(2) {
        return None;
    }
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).ok())
        .collect()
}

pub fn encrypt_val(plain: &str) -> String {
    if plain.starts_with("enc:") {
        return plain.to_string();
    }
    let key = get_machine_key();
    let bytes = plain.as_bytes();
    let mut encrypted = Vec::with_capacity(bytes.len());
    for (i, &b) in bytes.iter().enumerate() {
        let k = key[i % key.len()];
        let enc_byte = b ^ k ^ ((i * 7 + 13) as u8);
        encrypted.push(enc_byte);
    }
    format!("enc:{}", hex_encode(&encrypted))
}

pub fn decrypt_val(cipher: &str) -> String {
    if let Some(hex_part) = cipher.strip_prefix("enc:") {
        if let Ok(bytes) = hex_decode(hex_part).ok_or(()) {
            let key = get_machine_key();
            let mut decrypted = Vec::with_capacity(bytes.len());
            for (i, &b) in bytes.iter().enumerate() {
                let k = key[i % key.len()];
                let dec_byte = b ^ k ^ ((i * 7 + 13) as u8);
                decrypted.push(dec_byte);
            }
            if let Ok(s) = String::from_utf8(decrypted) {
                return s;
            }
        }
    }
    cipher.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt() {
        let secret = "ghp_1234567890abcdefghijklmnopqrst";
        let encrypted = encrypt_val(secret);

        assert!(encrypted.starts_with("enc:"));
        assert_ne!(secret, encrypted);

        let decrypted = decrypt_val(&encrypted);
        assert_eq!(secret, decrypted);
    }
}
