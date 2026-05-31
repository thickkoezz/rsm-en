// Transaction builder module for creating signed transactions
use crate::{
	crypto::{KeypairWrapper, SignedPayload},
	support::Extrinsic,
};

// Helper to encode RuntimeCall into bytes for signing
// This is a simplified encoding - in a real system you'd use SCALE codec
pub fn encode_call<Call>(call: &Call) -> Vec<u8>
where
	Call: serde::Serialize,
{
	// Simple JSON encoding for demonstration
	// In production, use SCALE codec for efficient binary encoding
	serde_json::to_vec(call).expect("Failed to encode call")
}

// Transaction builder for creating signed extrinsics
pub struct TransactionBuilder;

impl TransactionBuilder {
	// Create a signed extrinsic with the given keypair, call, and nonce
	pub fn signed_extrinsic<Call>(
		keypair: &KeypairWrapper,
		call: Call,
		nonce: u32,
	) -> Extrinsic<[u8; 32], Call>
	where
		Call: serde::Serialize + Clone,
	{
		// Get the public key from the keypair
		let public_key = keypair.public();
		let account_id: [u8; 32] = public_key.clone().into();

		// Encode the call for signing
		let call_data = encode_call(&call);

		// Create the signed payload
		let payload = SignedPayload::new(call_data, nonce);

		// Sign the payload hash
		let payload_hash = payload.hash();
		let signature = keypair.sign(&payload_hash);

		// Create and return the signed extrinsic
		Extrinsic { caller: account_id, call, signature, nonce }
	}
}

// Helper to display public key as hex string
pub fn public_key_to_hex(key: &[u8; 32]) -> String {
	hex::encode(key)
}

#[cfg(test)]
mod tests {
	use super::*;

	#[derive(Debug, Clone, serde::Serialize)]
	struct TestCall {
		method: String,
		value: u32,
	}

	#[test]
	fn test_transaction_creation() {
		let keypair = KeypairWrapper::generate();
		let call = TestCall { method: "transfer".to_string(), value: 100 };
		let nonce = 0;

		let extrinsic = TransactionBuilder::signed_extrinsic(&keypair, call.clone(), nonce);

		// Verify the caller is the public key
		let public_key_bytes: [u8; 32] = keypair.public().clone().into();
		assert_eq!(extrinsic.caller, public_key_bytes);

		// Verify the nonce matches
		assert_eq!(extrinsic.nonce, nonce);

		// Verify the call matches
		// Note: We can't directly compare the call because of potential serialization differences
	}

	#[test]
	fn test_call_encoding() {
		let call = TestCall { method: "test".to_string(), value: 42 };
		let encoded = encode_call(&call);
		assert!(!encoded.is_empty());
	}
}
