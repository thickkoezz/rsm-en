// Cryptographic primitives for signature verification and key management
use ed25519_dalek::{Signature, Signer, SigningKey, VerifyingKey};
use sha2::{Digest, Sha512};
use std::ops::Deref;

// Wrapper for ed25519 public key
#[derive(Debug, Clone, Eq)]
pub struct PublicKeyWrapper(pub VerifyingKey);

// Allow dereferencing to the underlying PublicKey
impl Deref for PublicKeyWrapper {
	type Target = VerifyingKey;
	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

// Convert from bytes
impl TryFrom<[u8; 32]> for PublicKeyWrapper {
	type Error = String;

	fn try_from(bytes: [u8; 32]) -> Result<Self, Self::Error> {
		VerifyingKey::from_bytes(&bytes)
			.map(PublicKeyWrapper)
			.map_err(|e| e.to_string())
	}
}

// Implement PartialEq manually
impl PartialEq for PublicKeyWrapper {
	fn eq(&self, other: &Self) -> bool {
		// Compare the byte representations
		self.0.to_bytes() == other.0.to_bytes()
	}
}

// Convert to bytes
impl From<PublicKeyWrapper> for [u8; 32] {
	fn from(key: PublicKeyWrapper) -> Self {
		key.0.to_bytes()
	}
}

// Wrapper for ed25519 signature
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignatureWrapper(pub Signature);

// Allow dereferencing to the underlying Signature
impl Deref for SignatureWrapper {
	type Target = Signature;
	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

// Convert from bytes
impl TryFrom<&[u8]> for SignatureWrapper {
	type Error = String;

	fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
		if bytes.len() != 64 {
			return Err("Invalid signature length".to_string());
		}
		let mut sig_bytes = [0u8; 64];
		sig_bytes.copy_from_slice(bytes);
		Signature::try_from(&sig_bytes).map(SignatureWrapper).map_err(|e| e.to_string())
	}
}

// Convert to bytes
impl From<SignatureWrapper> for [u8; 64] {
	fn from(sig: SignatureWrapper) -> Self {
		sig.0.to_bytes()
	}
}

// Convert from bytes array
impl TryFrom<[u8; 64]> for SignatureWrapper {
	type Error = String;

	fn try_from(bytes: [u8; 64]) -> Result<Self, Self::Error> {
		Signature::try_from(&bytes).map(SignatureWrapper).map_err(|e| e.to_string())
	}
}

// Wrapper for keypair
#[derive(Debug, Clone)]
pub struct KeypairWrapper(pub SigningKey);

// Allow dereferencing to the underlying Keypair
impl Deref for KeypairWrapper {
	type Target = SigningKey;
	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl KeypairWrapper {
	// Generate a new random keypair
	pub fn generate() -> Self {
		let mut rng = rand::rngs::OsRng::default();
		KeypairWrapper(SigningKey::generate(&mut rng))
	}

	// Get the public key
	pub fn public(&self) -> PublicKeyWrapper {
		PublicKeyWrapper(self.0.verifying_key())
	}

	// Sign a message
	pub fn sign(&self, message: &[u8]) -> SignatureWrapper {
		SignatureWrapper(self.0.sign(message))
	}
}

// Payload to be signed
#[derive(Debug, Clone)]
pub struct SignedPayload {
	// The call data encoded as bytes
	pub call_data: Vec<u8>,
	// The nonce for this transaction
	pub nonce: u32,
}

impl SignedPayload {
	// Create a new signed payload
	pub fn new(call_data: Vec<u8>, nonce: u32) -> Self {
		Self { call_data, nonce }
	}

	// Encode the payload for signing
	// We encode both the call data and nonce to prevent tampering
	pub fn encode(&self) -> Vec<u8> {
		let mut encoded = Vec::with_capacity(self.call_data.len() + 4);
		encoded.extend_from_slice(&self.call_data);
		encoded.extend_from_slice(&self.nonce.to_be_bytes());
		encoded
	}

	// Create a hash of the payload for signing
	pub fn hash(&self) -> [u8; 64] {
		let mut hasher = Sha512::new();
		hasher.update(&self.encode());
		hasher.finalize().into()
	}
}

// Verify a signature against a public key and message
pub fn verify(
	public_key: &PublicKeyWrapper,
	signature: &SignatureWrapper,
	message: &[u8],
) -> Result<(), String> {
	public_key
		.verify_strict(message, &signature.0)
		.map_err(|e| format!("Signature verification failed: {}", e))
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_keypair_generation() {
		let keypair = KeypairWrapper::generate();
		let public = keypair.public();
		// Public key should be 32 bytes
		assert_eq!(<[u8; 32]>::from(public).len(), 32);
	}

	#[test]
	fn test_signature_verification() {
		let keypair = KeypairWrapper::generate();
		let public = keypair.public();
		let message = b"test message";
		let signature = keypair.sign(message);

		// Valid signature should verify
		assert!(verify(&public, &signature, message).is_ok());

		// Wrong message should fail
		assert!(verify(&public, &signature, b"wrong message").is_err());
	}

	#[test]
	fn test_payload_encoding() {
		let call_data = vec![1, 2, 3, 4];
		let nonce = 42;
		let payload = SignedPayload::new(call_data.clone(), nonce);

		let encoded = payload.encode();
		assert_eq!(encoded.len(), call_data.len() + 4);
		assert_eq!(&encoded[..4], &call_data);
		assert_eq!(&encoded[4..], &nonce.to_be_bytes());
	}

	#[test]
	fn test_payload_hashing() {
		let payload = SignedPayload::new(vec![1, 2, 3], 100);
		let hash = payload.hash();
		// SHA512 produces 64 bytes
		assert_eq!(hash.len(), 64);
	}

	#[test]
	fn test_public_key_conversion() {
		let keypair = KeypairWrapper::generate();
		let public = keypair.public();
		let bytes: [u8; 32] = public.clone().into();
		let reconstructed = PublicKeyWrapper::try_from(bytes).unwrap();
		assert_eq!(public, reconstructed);
	}

	#[test]
	fn test_signature_conversion() {
		let keypair = KeypairWrapper::generate();
		let signature = keypair.sign(b"test");
		let bytes: [u8; 64] = signature.clone().into();
		let reconstructed = SignatureWrapper::try_from(bytes).unwrap();
		assert_eq!(signature, reconstructed);
	}
}
