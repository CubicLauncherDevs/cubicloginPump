use aes_gcm::aead::{Aead, KeyInit, Payload};
use aes_gcm::{Aes256Gcm, Key, Nonce};

const KEY_LENGTH: usize = 32;
const NONCE_LENGTH: usize = 12;

#[derive(Clone)]
pub struct Cipher {
    key: [u8; KEY_LENGTH],
}

impl Cipher {
    pub fn from_hex(encoded: &str) -> Result<Self, String> {
        let encoded = encoded.trim();
        if encoded.len() != KEY_LENGTH * 2 {
            return Err(format!(
                "the encryption key must be {} hex characters ({} bytes)",
                KEY_LENGTH * 2,
                KEY_LENGTH
            ));
        }

        let mut key = [0u8; KEY_LENGTH];
        for (index, byte) in key.iter_mut().enumerate() {
            let start = index * 2;
            *byte = u8::from_str_radix(&encoded[start..start + 2], 16)
                .map_err(|_| "the encryption key must be valid hexadecimal".to_string())?;
        }

        Ok(Self { key })
    }

    pub fn encrypt(&self, aad: &[u8], plaintext: &[u8]) -> Result<Vec<u8>, String> {
        let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&self.key));

        let mut nonce_bytes = [0u8; NONCE_LENGTH];
        getrandom::fill(&mut nonce_bytes)
            .map_err(|error| format!("failed to gather entropy: {error}"))?;

        let ciphertext = cipher
            .encrypt(
                Nonce::from_slice(&nonce_bytes),
                Payload {
                    msg: plaintext,
                    aad,
                },
            )
            .map_err(|_| "failed to encrypt record".to_string())?;

        let mut out = Vec::with_capacity(NONCE_LENGTH + ciphertext.len());
        out.extend_from_slice(&nonce_bytes);
        out.extend_from_slice(&ciphertext);
        Ok(out)
    }

    pub fn decrypt(&self, aad: &[u8], data: &[u8]) -> Result<Vec<u8>, String> {
        if data.len() <= NONCE_LENGTH {
            return Err("encrypted record is too short".to_string());
        }

        let (nonce_bytes, ciphertext) = data.split_at(NONCE_LENGTH);
        let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&self.key));

        cipher
            .decrypt(
                Nonce::from_slice(nonce_bytes),
                Payload {
                    msg: ciphertext,
                    aad,
                },
            )
            .map_err(|_| "failed to decrypt record".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cipher() -> Cipher {
        Cipher::from_hex(&"ab".repeat(32)).unwrap()
    }

    #[test]
    fn roundtrips() {
        let cipher = cipher();
        let data = cipher.encrypt(b"steve", b"secret payload").unwrap();
        assert_ne!(data, b"secret payload");
        assert_eq!(cipher.decrypt(b"steve", &data).unwrap(), b"secret payload");
    }

    #[test]
    fn aad_must_match() {
        let cipher = cipher();
        let data = cipher.encrypt(b"steve", b"payload").unwrap();
        assert!(cipher.decrypt(b"alex", &data).is_err());
    }

    #[test]
    fn rejects_bad_keys() {
        assert!(Cipher::from_hex("nope").is_err());
        assert!(Cipher::from_hex(&"zz".repeat(32)).is_err());
    }
}
