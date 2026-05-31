// Import standard library collections
use std::collections::BTreeMap;

// Import numeric traits for checked arithmetic operations
use num::traits::{CheckedAdd, CheckedSub, One, Zero};

// Configuration trait for the Balances pallet
// Must also implement the system::Config trait
pub trait Config: crate::system::Config {
	// Balance type must support zero, one, copying, checked addition, and checked subtraction
	type Balance: Zero + One + Copy + CheckedAdd + CheckedSub;
}

// Derive Debug and Clone traits for the Pallet struct
#[derive(Debug, Clone)]
// The Balances Pallet struct that manages account balances
// Generic over T which implements Config
pub struct Pallet<T: Config> {
	// Mapping from account ID to their balance
	balances: BTreeMap<T::AccountId, T::Balance>,
}

// Use the call procedural macro to generate dispatch code for callable functions
#[macros::call]
// Implementation of callable functions for the Balances Pallet
impl<T: Config> Pallet<T> {
	// Transfer tokens from caller to recipient
	// Returns Ok(()) on success, or an error message string on failure
	pub fn transfer(
		&mut self,
		// The account ID of the caller (sender)
		caller: T::AccountId,
		// The account ID of the recipient
		to: T::AccountId,
		// The amount to transfer
		amount: T::Balance,
	) -> Result<(), &'static str> {
		// Get the current balance of the caller
		let caller_balance = self.balance(&caller);
		// Get the current balance of the recipient
		let to_balance = self.balance(&to);

		// Calculate new caller balance with checked subtraction
		// Returns error if caller has insufficient funds
		let new_caller_balance = caller_balance.checked_sub(&amount).ok_or("Insufficient balance")?;

		// Calculate new recipient balance with checked addition
		// Returns error if addition would overflow
		let new_to_balance =
			to_balance.checked_add(&amount).ok_or("Overflow when adding to balance")?;

		// Update the caller's balance to the new (lower) amount
		self.set_balance(caller, new_caller_balance);
		// Update the recipient's balance to the new (higher) amount
		self.set_balance(to, new_to_balance);

		// Note: Event emission will be handled at the runtime level

		// Return success
		Ok(())
	}
}

// Implementation of non-callable functions for the Balances Pallet
impl<T: Config> Pallet<T> {
	// Create a new Balances Pallet with initial state
	pub fn new() -> Self {
		// Return a new Pallet with an empty balances map
		Self { balances: BTreeMap::new() }
	}

	// Set the balance of an account to a specific amount
	pub fn set_balance(&mut self, who: T::AccountId, amount: T::Balance) {
		// Insert or update the balance in the map
		self.balances.insert(who, amount);
	}

	// Get the current balance of an account
	pub fn balance(&mut self, who: &T::AccountId) -> T::Balance {
		// Return the balance if it exists, otherwise return zero
		*self.balances.get(who).unwrap_or(&T::Balance::zero())
	}
}

// Test module for Balances Pallet
#[cfg(test)]
mod test {
	// Import the system module for the Config trait
	use crate::system;

	// Test configuration struct for unit tests
	struct TestConfig;

	// Implement system::Config trait for TestConfig
	impl system::Config for TestConfig {
		// Use String as AccountId for testing
		type AccountId = String;
		// Use u32 as BlockNumber for testing
		type BlockNumber = u32;
		// Use u32 as Nonce for testing
		type Nonce = u32;
	}

	// Implement Balances::Config trait for TestConfig
	impl super::Config for TestConfig {
		// Use u128 as Balance for testing (large number support)
		type Balance = u128;
	}

	// Test that a new balances pallet starts with zero balance
	#[test]
	fn init_balance() {
		// Create a mutable Balances Pallet instance for testing
		let mut balances: super::Pallet<TestConfig> = super::Pallet::new();
		// Assert that Alice starts with 0 balance
		assert_eq!(balances.balance(&"alice".to_string()), 0);
		// Set Alice's balance to 100
		balances.set_balance("alice".to_string(), 100);
		// Assert that Alice's balance is now 100
		assert_eq!(balances.balance(&"alice".to_string()), 100);
	}

	// Test that balance transfer works correctly
	#[test]
	fn transfer_balance() {
		// Create test accounts
		let alice = "alice".to_string();
		let bob = "bob".to_string();

		// Create a mutable Balances Pallet instance for testing
		let mut balances: super::Pallet<TestConfig> = super::Pallet::new();
		// Set Alice's initial balance to 100
		balances.set_balance("alice".to_string(), 100);
		// Assert Alice has 100 and Bob has 0
		assert_eq!(balances.balance(&alice), 100);
		assert_eq!(balances.balance(&bob), 0);
		// Transfer 30 from Alice to Bob
		let _ = balances.transfer(alice.clone(), bob.clone(), 30);
		// Assert Alice now has 70 and Bob has 30
		assert_eq!(balances.balance(&alice), 70);
		assert_eq!(balances.balance(&bob), 30);
	}

	// Test that transfer fails with insufficient balance
	#[test]
	fn transfer_balance_insufficient() {
		// Create a mutable Balances Pallet instance for testing
		let mut balances: super::Pallet<TestConfig> = super::Pallet::new();
		// Set Alice's balance to 100
		balances.set_balance("alice".to_string(), 100);
		// Set Bob's balance to maximum u128 value
		balances.set_balance("bob".to_string(), u128::MAX);

		// Try to transfer more than Alice has
		let result: Result<(), &str> =
			balances.transfer("alice".to_string(), "bob".to_string(), u128::MAX);
		// Assert the transfer failed with insufficient balance error
		assert_eq!(result, Err("Insufficient balance"));
		// Assert balances remain unchanged
		assert_eq!(balances.balance(&"alice".to_string()), 100);
		assert_eq!(balances.balance(&"bob".to_string()), u128::MAX);
	}

	// Test that transfer fails on overflow
	#[test]
	fn transfer_balance_overflow() {
		// Create a mutable Balances Pallet instance for testing
		let mut balances: super::Pallet<TestConfig> = super::Pallet::new();
		// Set Alice's balance to maximum u128 value
		balances.set_balance("alice".to_string(), u128::MAX);
		// Set Bob's balance to maximum u128 value
		balances.set_balance("bob".to_string(), u128::MAX);

		// Try to transfer from Alice to Bob (would overflow Bob's balance)
		let result = balances.transfer("alice".to_string(), "bob".to_string(), u128::MAX);
		// Assert the transfer failed with overflow error
		assert_eq!(result, Err("Overflow when adding to balance"));
		// Assert balances remain unchanged
		assert_eq!(balances.balance(&"alice".to_string()), u128::MAX);
		assert_eq!(balances.balance(&"bob".to_string()), u128::MAX);
	}
}
