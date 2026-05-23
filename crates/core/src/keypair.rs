use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;
use ed25519_dalek::{Signature as DalekSignature, SigningKey, VerifyingKey};
use ed25519_dalek::Signer;
use ed25519_dalek::Verifier;
use rand::rngs::OsRng;
use rand::RngCore;

use crate::error::{Error, Result};
use crate::types::{KeyPair, PqcSignature, PublicKey, SecretKey, Signature};

pub trait KeyPairExt {
    fn generate() -> KeyPair;
    fn from_secret(secret: &[u8]) -> Result<KeyPair>;
    fn sign(&self, data: &[u8]) -> Signature;
    fn verify(&self, data: &[u8], sig: &Signature) -> bool;
    fn pqc_sign(&self, data: &[u8]) -> Result<PqcSignature>;
    fn pqc_verify(&self, data: &[u8], sig: &PqcSignature) -> Result<bool>;
    fn to_pem(&self) -> String;
    fn from_pem(pem: &str) -> Result<KeyPair>;
    fn public_key_bytes(&self) -> Result<[u8; 32]>;
    fn secret_key_bytes(&self) -> Result<[u8; 32]>;
}

impl KeyPairExt for KeyPair {
    fn generate() -> Self {
        let mut seed = [0u8; 32];
        OsRng.fill_bytes(&mut seed);
        let signing_key = SigningKey::from_bytes(&seed);
        let verifying_key = signing_key.verifying_key();
        KeyPair {
            public: PublicKey(BASE64.encode(verifying_key.to_bytes())),
            secret: SecretKey(signing_key.to_bytes().to_vec()),
        }
    }

    fn from_secret(secret: &[u8]) -> Result<Self> {
        let bytes: [u8; 32] = secret
            .try_into()
            .map_err(|_| Error::InvalidKey("secret key must be 32 bytes".into()))?;
        let signing_key = SigningKey::from_bytes(&bytes);
        let verifying_key = signing_key.verifying_key();
        Ok(KeyPair {
            public: PublicKey(BASE64.encode(verifying_key.to_bytes())),
            secret: SecretKey(signing_key.to_bytes().to_vec()),
        })
    }

    fn sign(&self, data: &[u8]) -> Signature {
        let bytes = self
            .secret_key_bytes()
            .expect("valid 32-byte secret key");
        let signing_key = SigningKey::from_bytes(&bytes);
        let dalek_sig = signing_key.sign(data);
        Signature(BASE64.encode(dalek_sig.to_bytes()))
    }

    fn verify(&self, data: &[u8], sig: &Signature) -> bool {
        let pk_bytes = match self.public_key_bytes() {
            Ok(b) => b,
            Err(_) => return false,
        };
        let verifying_key = match VerifyingKey::from_bytes(&pk_bytes) {
            Ok(vk) => vk,
            Err(_) => return false,
        };
        let sig_bytes: [u8; 64] = match BASE64.decode(&sig.0) {
            Ok(v) => match v.try_into() {
                Ok(b) => b,
                Err(_) => return false,
            },
            Err(_) => return false,
        };
        let dalek_sig = DalekSignature::from_bytes(&sig_bytes);
        verifying_key.verify(data, &dalek_sig).is_ok()
    }

    #[cfg(feature = "pqc")]
    fn pqc_sign(&self, data: &[u8]) -> Result<PqcSignature> {
        use liboqs::sig::*;
        let mut signer = Sig::new(SigAlg::Dilithium5)
            .map_err(|e| Error::PqcError(format!("failed to create signer: {}", e)))?;
        let secret_bytes = self.secret_key_bytes()?;
        signer
            .set_secret_key(&secret_bytes)
            .map_err(|e| Error::PqcError(format!("failed to set secret key: {}", e)))?;
        let signature = signer
            .sign(data)
            .map_err(|e| Error::PqcError(format!("signing failed: {}", e)))?;
        Ok(PqcSignature(BASE64.encode(signature)))
    }

    #[cfg(not(feature = "pqc"))]
    fn pqc_sign(&self, _data: &[u8]) -> Result<PqcSignature> {
        Err(Error::PqcError(
            "PQC support not enabled (compile with feature 'pqc')".into(),
        ))
    }

    #[cfg(feature = "pqc")]
    fn pqc_verify(&self, data: &[u8], sig: &PqcSignature) -> Result<bool> {
        use liboqs::sig::*;
        let mut verifier = Sig::new(SigAlg::Dilithium5)
            .map_err(|e| Error::PqcError(format!("failed to create verifier: {}", e)))?;
        let pk_bytes = self.public_key_bytes()?;
        verifier
            .set_public_key(&pk_bytes)
            .map_err(|e| Error::PqcError(format!("failed to set public key: {}", e)))?;
        let sig_bytes = BASE64
            .decode(&sig.0)
            .map_err(|e| Error::PqcError(format!("invalid base64: {}", e)))?;
        match verifier.verify(data, &sig_bytes) {
            Ok(_) => Ok(true),
            Err(liboqs::sig::Error::VerificationFailed) => Ok(false),
            Err(e) => Err(Error::PqcError(format!("verification error: {}", e))),
        }
    }

    #[cfg(not(feature = "pqc"))]
    fn pqc_verify(&self, _data: &[u8], _sig: &PqcSignature) -> Result<bool> {
        Err(Error::PqcError(
            "PQC support not enabled (compile with feature 'pqc')".into(),
        ))
    }

    fn to_pem(&self) -> String {
        let b64 = BASE64.encode(&self.secret.0);
        let mut pem = String::from("-----BEGIN POI PRIVATE KEY-----\n");
        for chunk in b64.as_bytes().chunks(64) {
            pem.push_str(&String::from_utf8_lossy(chunk));
            pem.push('\n');
        }
        pem.push_str("-----END POI PRIVATE KEY-----\n");
        pem
    }

    fn from_pem(pem: &str) -> Result<KeyPair> {
        let pem = pem.trim();
        if !pem.starts_with("-----BEGIN POI PRIVATE KEY-----")
            || !pem.ends_with("-----END POI PRIVATE KEY-----")
        {
            return Err(Error::InvalidKey("invalid PEM format".into()));
        }
        let b64: String = pem
            .lines()
            .skip(1)
            .take_while(|l| !l.starts_with("-----END"))
            .collect();
        let secret = BASE64
            .decode(b64.trim())
            .map_err(|e| Error::InvalidKey(format!("base64 decode error: {}", e)))?;
        KeyPair::from_secret(&secret)
    }

    fn public_key_bytes(&self) -> Result<[u8; 32]> {
        let bytes = BASE64
            .decode(&self.public.0)
            .map_err(|e| Error::InvalidKey(format!("invalid public key base64: {}", e)))?;
        bytes
            .try_into()
            .map_err(|_| Error::InvalidKey("public key must be 32 bytes".into()))
    }

    fn secret_key_bytes(&self) -> Result<[u8; 32]> {
        self.secret
            .0
            .as_slice()
            .try_into()
            .map_err(|_| Error::InvalidKey("secret key must be 32 bytes".into()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keypair_generate() {
        let kp = KeyPair::generate();
        assert!(!kp.public.0.is_empty());
        assert_eq!(kp.secret.0.len(), 32);
    }

    #[test]
    fn test_sign_verify() {
        let kp = KeyPair::generate();
        let data = b"hello world";
        let sig = kp.sign(data);
        assert!(kp.verify(data, &sig));
    }

    #[test]
    fn test_verify_invalid_signature() {
        let kp = KeyPair::generate();
        let data = b"hello world";
        let sig = kp.sign(data);
        let wrong_data = b"wrong data";
        assert!(!kp.verify(wrong_data, &sig));
    }

    #[test]
    fn test_from_secret() {
        let kp1 = KeyPair::generate();
        let kp2 = KeyPair::from_secret(&kp1.secret.0).unwrap();
        assert_eq!(kp1.public.0, kp2.public.0);
        assert_eq!(kp1.secret.0, kp2.secret.0);
    }

    #[test]
    fn test_from_secret_invalid_length() {
        let result = KeyPair::from_secret(&[0u8; 16]);
        assert!(result.is_err());
    }

    #[test]
    fn test_pem_roundtrip() {
        let kp1 = KeyPair::generate();
        let pem = kp1.to_pem();
        let kp2 = KeyPair::from_pem(&pem).unwrap();
        assert_eq!(kp1.public.0, kp2.public.0);
        assert_eq!(kp1.secret.0, kp2.secret.0);
    }

    #[test]
    fn test_verify_with_other_keypair_fails() {
        let kp1 = KeyPair::generate();
        let kp2 = KeyPair::generate();
        let data = b"test data";
        let sig = kp1.sign(data);
        assert!(!kp2.verify(data, &sig));
    }

    #[test]
    fn test_pqc_sign_disabled_by_default() {
        let kp = KeyPair::generate();
        let result = kp.pqc_sign(b"test");
        assert!(result.is_err());
    }
}
