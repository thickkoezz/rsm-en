// Import numeric traits from the num crate for arithmetic operations
use num::traits::{One, Zero};
// Import standard library collections and operations
use std::{collections::BTreeMap, ops::AddAssign};

// Configuration trait that defines the types used by the System pallet
pub trait Config {
	// AccountId must support ordering (for BTreeMap) and be cloneable
	type AccountId: Ord + Clone;
	// BlockNumber must support zero, one, addition assignment, copying, and ordering
	type BlockNumber: Zero + One + AddAssign + Copy + Ord + PartialEq;
	// Nonce must support zero, one, copying, and equality comparison
	type Nonce: Zero + One + Copy + PartialEq;
}

// Derive Debug and Clone traits for the Pallet struct
#[derive(Debug, Clone)]
// The System Pallet struct that manages blockchain-wide state
// Generic over T which implements Config
pub struct Pallet<T: Config> {
	// Current block number in the chain
	pub block_number: T::BlockNumber,
	// Mapping from account ID to their nonce (transaction count)
	pub nonce: BTreeMap<T::AccountId, T::Nonce>,
}

// Implementation of System Pallet methods
impl<T: Config> Pallet<T> {
	// Create a new System Pallet with initial state
	pub fn new() -> Self {
		// Return a new Pallet with block_number set to zero and empty nonce map
		Self { block_number: T::BlockNumber::zero(), nonce: BTreeMap::new() }
	}

	// Get the current block number
	pub fn block_number(&self) -> T::BlockNumber {
		// Return the current block number
		self.block_number
	}

	// Increment the block number by one
	pub fn inc_block_number(&mut self) {
		// This version intentionally crashes on overflow for simplicity
		// self.block_number = self.block_number.checked_add(&BlockNumber::one()).unwrap()
		// Add one to the current block number (will panic on overflow)
		self.block_number += T::BlockNumber::one();
	}

	// Increment the nonce for a specific account
	pub fn inc_nonce(&mut self, who: &T::AccountId) {
		// Get the current nonce for the account, defaulting to zero if not found
		let nonce = *self.nonce.get(who).unwrap_or(&T::Nonce::zero());
		// Insert the incremented nonce back into the map
		self.nonce.insert(who.clone(), nonce + T::Nonce::one());
	}

	// Get the current nonce for a specific account
	pub fn get_nonce(&self, who: &T::AccountId) -> T::Nonce {
		// Return the nonce for the account, defaulting to zero if not found
		*self.nonce.get(who).unwrap_or(&T::Nonce::zero())
	}

	// Verify that the nonce matches the expected value for replay protection
	// This must be checked BEFORE incrementing the nonce
	pub fn verify_nonce(
		&self,
		who: &T::AccountId,
		expected_nonce: T::Nonce,
	) -> Result<(), &'static str> {
		let current_nonce = self.get_nonce(who);
		if current_nonce != expected_nonce {
			return Err("Nonce mismatch - transaction may be replayed or out of order");
		}
		Ok(())
	}
}

// Test module for System Pallet
#[cfg(test)]
mod test {
	// Test configuration struct for unit tests
	struct TestConfig;

	// Implement Config trait for TestConfig
	impl super::Config for TestConfig {
		// Use String as AccountId for testing
		type AccountId = String;
		// Use u32 as BlockNumber for testing
		type BlockNumber = u32;
		// Use u32 as Nonce for testing
		type Nonce = u32;
	}

	// Test that system initializes with block number 0
	#[test]
	fn init_system() {
		// Create a new System Pallet instance for testing
		let system: super::Pallet<TestConfig> = super::Pallet::new();
		// Assert that the initial block number is 0
		assert_eq!(system.block_number, 0);
	}

	// Test that block number increments correctly
	#[test]
	fn inc_block_number() {
		// Create a mutable System Pallet instance for testing
		let mut system: super::Pallet<TestConfig> = super::Pallet::new();
		// Increment the block number
		system.inc_block_number();
		// Assert that the block number is now 1
		assert_eq!(system.block_number, 1);
	}

	// Test that nonce increments correctly for an account
	#[test]
	fn inc_nonce() {
		// Create a test account named "alice"
		let alice = String::from("alice");
		// Create a mutable System Pallet instance for testing
		let mut system: super::Pallet<TestConfig> = super::Pallet::new();
		// Increment Alice's nonce twice
		system.inc_nonce(&alice.clone());
		system.inc_nonce(&alice.clone());
		// Assert that Alice's nonce is now 2
		assert_eq!(system.get_nonce(&alice), 2);
	}
}
