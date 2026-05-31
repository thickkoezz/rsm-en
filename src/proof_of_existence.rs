// Import the DispatchResult type alias for return types
use crate::support::DispatchResult;
// Import core fmt Debug trait for debugging
use core::fmt::Debug;
// Import standard library collections
use std::collections::BTreeMap;

// Configuration trait for the Proof of Existence pallet
// Must also implement the system::Config trait
pub trait Config: crate::system::Config {
	// Content type must support debugging, ordering, and cloning
	type Content: Debug + Ord + Clone;
}

// Derive Debug and Clone traits for the Pallet struct
#[derive(Debug, Clone)]
// The Proof of Existence Pallet struct that manages claims
// Generic over T which implements Config
pub struct Pallet<T: Config> {
	// Mapping from content to the account that owns the claim
	pub claims: BTreeMap<T::Content, T::AccountId>,
}

// Use the call procedural macro to generate dispatch code for callable functions
#[macros::call]
// Implementation of callable functions for the Proof of Existence Pallet
impl<T: Config> Pallet<T> {
	// Create a new claim for content, associating it with the caller
	// Returns Ok(()) on success, or an error message string on failure
	pub fn create_claim(&mut self, caller: T::AccountId, claim: T::Content) -> DispatchResult {
		// Check if the claim already exists
		match self.get_claim(&claim) {
			// If claim exists, return an error
			Some(_) => Err("Claim already exists"),
			// If claim doesn't exist, create it
			None => {
				// Insert the claim with the caller as the owner
				self.claims.insert(claim.clone(), caller.clone());
				// Note: Event emission will be handled at the runtime level
				// Return success
				Ok(())
			},
		}
	}

	// Revoke an existing claim, only allowed by the claim owner
	// Returns Ok(()) on success, or an error message string on failure
	pub fn revoke_claim(&mut self, caller: T::AccountId, claim: T::Content) -> DispatchResult {
		// Try to get the owner of the claim, return error if claim doesn't exist
		let claim_owner = self.get_claim(&claim).ok_or("Claim does not exist")?;

		// Check if the caller is the owner of the claim
		if claim_owner != &caller {
			// Return error if caller is not the owner
			return Err("Caller is not the owner of the claim");
		}

		// Remove the claim from the map
		self.claims.remove(&claim);
		// Note: Event emission will be handled at the runtime level

		// Return success
		Ok(())
	}
}

// Implementation of non-callable functions for the Proof of Existence Pallet
impl<T: Config> Pallet<T> {
	// Create a new Proof of Existence Pallet with initial state
	pub fn new() -> Self {
		// Return a new Pallet with an empty claims map
		Self { claims: BTreeMap::new() }
	}

	// Get the owner of a claim if it exists
	pub fn get_claim(&self, claim: &T::Content) -> Option<&T::AccountId> {
		// Return the owner of the claim if it exists, otherwise None
		self.claims.get(claim)
	}
}

// Test module for Proof of Existence Pallet
#[cfg(test)]
mod test {
	// Test configuration struct for unit tests
	struct TestConfig;

	// Implement Proof of Existence::Config trait for TestConfig
	impl super::Config for TestConfig {
		// Use static string slices as Content for testing
		type Content = &'static str;
	}

	// Implement system::Config trait for TestConfig
	impl crate::system::Config for TestConfig {
		// Use static string slices as AccountId for testing
		type AccountId = &'static str;
		// Use u32 as BlockNumber for testing
		type BlockNumber = u32;
		// Use u32 as Nonce for testing
		type Nonce = u32;
	}

	// Test basic proof of existence functionality
	#[test]
	fn basic_proof_of_existence() {
		// Create a mutable Proof of Existence Pallet instance for testing
		let mut poe = super::Pallet::<TestConfig>::new();

		// Alice creates a claim for "my_document"
		let _ = poe.create_claim("alice", "my_document");
		// Assert that Alice is the owner of "my_document"
		assert_eq!(poe.get_claim(&"my_document"), Some(&"alice"));

		// Bob tries to revoke Alice's claim (should fail)
		let res = poe.revoke_claim("bob", "my_document");
		// Assert the revocation failed because Bob is not the owner
		assert_eq!(res, Err("Caller is not the owner of the claim"));

		// Bob tries to create a claim for "my_document" (should fail, already exists)
		let res = poe.create_claim("bob", "my_document");
		// Assert the creation failed because claim already exists
		assert_eq!(res, Err("Claim already exists"));

		// Bob tries to revoke a non-existent claim (should fail)
		let res = poe.revoke_claim("bob", "non_existent");
		// Assert the revocation failed because claim doesn't exist
		assert_eq!(res, Err("Claim does not exist"));

		// Alice revokes her own claim (should succeed)
		let res = poe.revoke_claim("alice", "my_document");
		// Assert the revocation succeeded
		assert_eq!(res, Ok(()));
		// Assert the claim no longer exists
		assert_eq!(poe.get_claim(&"my_document"), None);
	}
}
