// Import numeric traits for checked arithmetic operations
use num::traits::{CheckedAdd, CheckedSub, Zero};

// Configuration trait for the Fees pallet
// Must implement system::Config and balances::Config traits
pub trait Config: crate::system::Config + crate::balances::Config {
	// The flat fee charged per transaction
	// This is a compile-time constant that must be specified by the runtime
	const FEE: Self::Balance;
}

// Derive Debug and Clone traits for the Pallet struct
#[derive(Debug, Clone)]
// The Fees Pallet struct that manages transaction fee collection
// Generic over T which implements Config
pub struct Pallet<T: Config> {
	// Track total fees collected (for analytics/monitoring)
	total_fees_collected: T::Balance,
}

// Callable interface for the Fees Pallet
// Note: This pallet is not directly user-callable, but used by the runtime during execution
impl<T: Config> Pallet<T> {
	// Calculate the fee for a transaction (flat fee model)
	// Returns the configured flat fee amount
	pub fn calculate_fee(&self) -> T::Balance {
		T::FEE
	}

	// Pay fee from caller's balance
	// This deducts the fee from the caller's balance and tracks total fees collected
	//
	// Parameters:
	// - balances_pallet: Mutable reference to the balances pallet to update balances
	// - caller: The account ID of the fee payer
	//
	// Returns:
	// - Ok(()) if fee was successfully paid
	// - Err with message if caller has insufficient balance or arithmetic error occurs
	pub fn pay_fee(
		&mut self,
		balances_pallet: &mut crate::balances::Pallet<T>,
		caller: T::AccountId,
	) -> Result<(), &'static str>
	where
		T::Balance: core::cmp::PartialOrd,
	{
		// Calculate the fee to be paid
		let fee = self.calculate_fee();
		// Get the caller's current balance
		let caller_balance = balances_pallet.balance(&caller);

		// Check if caller has sufficient balance to pay the fee
		if caller_balance < fee {
			return Err("Insufficient balance to pay fee");
		}

		// Deduct the fee from caller's balance with checked subtraction
		let new_balance = caller_balance.checked_sub(&fee).ok_or("Underflow when deducting fee")?;

		// Update the caller's balance
		balances_pallet.set_balance(caller, new_balance);

		// Add the fee to total fees collected with checked addition
		self.total_fees_collected = self
			.total_fees_collected
			.checked_add(&fee)
			.ok_or("Overflow when adding to total fees")?;

		Ok(())
	}

	// Get the total fees collected so far
	// This is useful for analytics and monitoring
	pub fn total_fees_collected(&self) -> T::Balance {
		self.total_fees_collected
	}
}

// Implementation of non-callable functions for the Fees Pallet
impl<T: Config> Pallet<T> {
	// Create a new Fees Pallet with initial state
	pub fn new() -> Self {
		// Return a new Pallet with zero fees collected
		Self { total_fees_collected: T::Balance::zero() }
	}
}

// Test module for Fees Pallet
#[cfg(test)]
mod test {
	// Import the system and balances modules for the Config traits
	use crate::{balances, system};

	// Test configuration struct for unit tests
	struct TestConfig;

	// Implement system::Config trait for TestConfig
	impl system::Config for TestConfig {
		type AccountId = String;
		type BlockNumber = u32;
		type Nonce = u32;
	}

	// Implement balances::Config trait for TestConfig
	impl balances::Config for TestConfig {
		type Balance = u128;
	}

	// Implement Fees::Config trait for TestConfig
	impl super::Config for TestConfig {
		const FEE: u128 = 1; // 1 token fee for testing
	}

	// Test that calculate_fee returns the configured FEE
	#[test]
	fn calculate_fee() {
		let fees: super::Pallet<TestConfig> = super::Pallet::new();
		assert_eq!(fees.calculate_fee(), 1);
	}

	// Test successful fee payment
	#[test]
	fn pay_fee_success() {
		// Create fees and balances pallets
		let mut fees: super::Pallet<TestConfig> = super::Pallet::new();
		let mut balances: crate::balances::Pallet<TestConfig> = crate::balances::Pallet::new();

		// Set up Alice with 100 tokens
		let alice = "alice".to_string();
		balances.set_balance(alice.clone(), 100);

		// Pay fee
		let result = fees.pay_fee(&mut balances, alice.clone());

		// Assert fee payment succeeded
		assert_eq!(result, Ok(()));
		// Assert Alice's balance decreased by fee
		assert_eq!(balances.balance(&alice), 99);
		// Assert total fees collected increased
		assert_eq!(fees.total_fees_collected(), 1);
	}

	// Test insufficient balance for fee
	#[test]
	fn pay_fee_insufficient() {
		// Create fees and balances pallets
		let mut fees: super::Pallet<TestConfig> = super::Pallet::new();
		let mut balances: crate::balances::Pallet<TestConfig> = crate::balances::Pallet::new();

		// Set up Alice with 0 tokens
		let alice = "alice".to_string();
		balances.set_balance(alice.clone(), 0);

		// Try to pay fee
		let result = fees.pay_fee(&mut balances, alice.clone());

		// Assert fee payment failed
		assert_eq!(result, Err("Insufficient balance to pay fee"));
		// Assert Alice's balance remains 0
		assert_eq!(balances.balance(&alice), 0);
		// Assert total fees collected remains 0
		assert_eq!(fees.total_fees_collected(), 0);
	}

	// Test that total fees accumulate correctly
	#[test]
	fn total_fees_accumulation() {
		// Create fees and balances pallets
		let mut fees: super::Pallet<TestConfig> = super::Pallet::new();
		let mut balances: crate::balances::Pallet<TestConfig> = crate::balances::Pallet::new();

		// Set up Alice and Bob with tokens
		let alice = "alice".to_string();
		let bob = "bob".to_string();
		balances.set_balance(alice.clone(), 100);
		balances.set_balance(bob.clone(), 50);

		// Both pay fees
		let _ = fees.pay_fee(&mut balances, alice.clone());
		let _ = fees.pay_fee(&mut balances, bob.clone());

		// Assert total fees collected is 2 (1 from each)
		assert_eq!(fees.total_fees_collected(), 2);
		// Assert balances are correct
		assert_eq!(balances.balance(&alice), 99);
		assert_eq!(balances.balance(&bob), 49);
	}
}
