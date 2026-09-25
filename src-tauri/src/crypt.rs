use aes_gcm::aead::Aead;
use aes_gcm::aead::OsRng;
use aes_gcm::{AeadCore, Aes256Gcm, KeyInit, Nonce};
use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use sha2::{Digest, Sha256};

const PREFIX: &str = "hataclip1.";

pub(crate) fn seal(plain: &str, key: &str) -> Option<String> {
    if key.is_empty() {
        return None;
    }
    let cipher = Aes256Gcm::new_from_slice(&Sha256::digest(key.as_bytes())).ok()?;
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
    let mut packed = nonce.to_vec();
    packed.extend(cipher.encrypt(&nonce, plain.as_bytes()).ok()?);
    Some(format!("{PREFIX}{}", STANDARD.encode(packed)))
}

pub(crate) fn open(packed: &str, key: &str) -> Option<String> {
    if key.is_empty() {
        return None;
    }
    let body = packed.trim().strip_prefix(PREFIX)?;
    let bytes = STANDARD.decode(body).ok()?;
    if bytes.len() < 12 + 16 {
        return None;
    }
    let (nonce, ciphertext) = bytes.split_at(12);
    let cipher = Aes256Gcm::new_from_slice(&Sha256::digest(key.as_bytes())).ok()?;
    let plain = cipher
        .decrypt(Nonce::from_slice(nonce), ciphertext)
        .ok()?;
    String::from_utf8(plain).ok()
}

pub(crate) fn apply(kind: &str, text: &str, key: &str) -> Option<String> {
    match kind {
        "crypt" => seal(text, key),
        "decrypt" => open(text, key),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_and_wrong_key_does_nothing() {
        let sealed = seal("hello", "secret").unwrap();
        assert!(sealed.starts_with(PREFIX));
        assert!(!sealed.contains("secret"));
        assert!(!sealed.contains("hello"));
        assert_eq!(open(&sealed, "secret").as_deref(), Some("hello"));
        assert!(open(&sealed, "other").is_none());
        assert!(open("hello", "secret").is_none());
        assert!(seal("hello", "").is_none());
        assert!(open(&sealed, "").is_none());
        let again = seal("hello", "secret").unwrap();
        assert_ne!(sealed, again);
        assert_eq!(open(&again, "secret").as_deref(), Some("hello"));
    }
}
