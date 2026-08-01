// ---------------------------------------------------------------------------
//  Thunder Blockchain — Core Cryptographic Primitives
// ---------------------------------------------------------------------------
//  Provides SHA-256 hashing, Ed25519 key-pair generation, digital signatures,
//  and address derivation used throughout the entire blockchain.
// ---------------------------------------------------------------------------

use ed25519_dalek::{Signer, SigningKey, Verifier, VerifyingKey};
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fmt;

// ── Types ──────────────────────────────────────────────────────────────────

/// A 32-byte SHA-256 hash.
pub type Hash = [u8; 32];

/// A 20-byte address derived from a public key.
pub type Address = [u8; 20];

/// A 64-byte Ed25519 signature.
pub type Signature = [u8; 64];

/// A 32-byte Ed25519 public key.
pub type PublicKey = [u8; 32];

// ── Hashing ────────────────────────────────────────────────────────────────

/// Compute the SHA-256 hash of arbitrary data.
pub fn hash_sha256(data: &[u8]) -> Hash {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let result = hasher.finalize();
    let mut hash = [0u8; 32];
    hash.copy_from_slice(&result);
    hash
}

/// Compute a double SHA-256 hash (hash of hash) for extra security.
pub fn double_hash(data: &[u8]) -> Hash {
    hash_sha256(&hash_sha256(data))
}

// ── Key Pair ───────────────────────────────────────────────────────────────

/// An Ed25519 key pair for signing and verification.
#[derive(Clone)]
pub struct KeyPair {
    signing_key: SigningKey,
}

impl KeyPair {
    /// Generate a new random key pair.
    pub fn generate() -> Self {
        let signing_key = SigningKey::generate(&mut OsRng);
        Self { signing_key }
    }

    /// Restore a key pair from raw secret key bytes (32 bytes).
    pub fn from_secret_bytes(bytes: &[u8; 32]) -> Self {
        let signing_key = SigningKey::from_bytes(bytes);
        Self { signing_key }
    }

    /// Return the raw secret key bytes.
    pub fn secret_bytes(&self) -> &[u8; 32] {
        self.signing_key.as_bytes()
    }

    /// Return the 32-byte public key.
    pub fn public_key(&self) -> PublicKey {
        let vk = self.signing_key.verifying_key();
        vk.to_bytes()
    }

    /// Derive a 20-byte address from the public key.
    pub fn address(&self) -> Address {
        address_from_public_key(&self.public_key())
    }

    /// Sign arbitrary data, returning a 64-byte signature.
    pub fn sign(&self, data: &[u8]) -> Signature {
        let sig = self.signing_key.sign(data);
        sig.to_bytes()
    }
}

// ── Standalone helpers ─────────────────────────────────────────────────────

/// Derive a 20-byte address from a 32-byte public key (first 20 bytes of its
/// SHA-256 hash).
pub fn address_from_public_key(public_key: &PublicKey) -> Address {
    let hash = hash_sha256(public_key);
    let mut addr = [0u8; 20];
    addr.copy_from_slice(&hash[..20]);
    addr
}

/// Verify an Ed25519 signature against a public key and message.
pub fn verify_signature(public_key: &PublicKey, message: &[u8], signature: &Signature) -> bool {
    let Ok(vk) = VerifyingKey::from_bytes(public_key) else {
        return false;
    };
    let Ok(sig) = ed25519_dalek::Signature::from_slice(signature) else {
        return false;
    };
    vk.verify(message, &sig).is_ok()
}

// ── Display helpers (hex) ──────────────────────────────────────────────────

/// Pretty-print a hash as a hex string.
pub fn hash_to_hex(hash: &Hash) -> String {
    hex::encode(hash)
}

/// Pretty-print an address as a hex string prefixed with `0x`.
pub fn address_to_hex(addr: &Address) -> String {
    format!("0x{}", hex::encode(addr))
}

/// Parse a hex string (with optional `0x` prefix) into an `Address`.
pub fn address_from_hex(s: &str) -> Result<Address, hex::FromHexError> {
    let s = s.strip_prefix("0x").unwrap_or(s);
    let bytes = hex::decode(s)?;
    let mut addr = [0u8; 20];
    addr.copy_from_slice(&bytes[..20]);
    Ok(addr)
}

// ── SerializableKeyPair (for JSON export) ──────────────────────────────────

/// A serialisable representation of a key pair (for wallet export / import).
#[derive(Serialize, Deserialize)]
pub struct SerializableKeyPair {
    pub secret_key: String,
    pub public_key: String,
    pub address: String,
}

impl From<&KeyPair> for SerializableKeyPair {
    fn from(kp: &KeyPair) -> Self {
        Self {
            secret_key: hex::encode(kp.secret_bytes()),
            public_key: hex::encode(kp.public_key()),
            address: address_to_hex(&kp.address()),
        }
    }
}

// ── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_deterministic() {
        let a = hash_sha256(b"thunder");
        let b = hash_sha256(b"thunder");
        assert_eq!(a, b);
    }

    #[test]
    fn test_hash_different_inputs() {
        let a = hash_sha256(b"hello");
        let b = hash_sha256(b"world");
        assert_ne!(a, b);
    }

    #[test]
    fn test_keypair_sign_verify() {
        let kp = KeyPair::generate();
        let msg = b"Thunder Blockchain transaction";
        let sig = kp.sign(msg);
        assert!(verify_signature(&kp.public_key(), msg, &sig));
    }

    #[test]
    fn test_invalid_signature() {
        let kp1 = KeyPair::generate();
        let kp2 = KeyPair::generate();
        let msg = b"Thunder Blockchain transaction";
        let sig = kp1.sign(msg);
        // Signature from kp1 should NOT verify under kp2's public key
        assert!(!verify_signature(&kp2.public_key(), msg, &sig));
    }

    #[test]
    fn test_address_derivation() {
        let kp = KeyPair::generate();
        let addr = kp.address();
        assert_eq!(addr.len(), 20);
        assert_eq!(addr, address_from_public_key(&kp.public_key()));
    }

    #[test]
    fn test_keypair_restore_from_secret() {
        let kp = KeyPair::generate();
        let restored = KeyPair::from_secret_bytes(kp.secret_bytes());
        assert_eq!(kp.public_key(), restored.public_key());
        assert_eq!(kp.address(), restored.address());
    }

    #[test]
    fn test_address_hex_roundtrip() {
        let kp = KeyPair::generate();
        let hex_str = address_to_hex(&kp.address());
        let parsed = address_from_hex(&hex_str).unwrap();
        assert_eq!(kp.address(), parsed);
    }
}
