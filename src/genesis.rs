//! Genesis Configuration Module
//!
//! This module provides a formal way to configure the initial state of the blockchain.
//! Instead of manually calling `set_balance()` and other initialization functions,
//! you can define a `GenesisConfig` struct that captures all initial state.

use std::collections::BTreeMap;

// Import the core types from the parent crate
use crate::types::{AccountId, Balance, BlockNumber, Content, Nonce};

/// Genesis configuration for the blockchain
///
/// This struct defines all the initial state when creating a new blockchain runtime.
/// It allows for a declarative way to set up initial balances, claims, and other state.
#[derive(Debug, Clone)]
pub struct GenesisConfig {
	/// Initial account balances
	/// Maps account IDs to their starting balance
	pub balances: BTreeMap<AccountId, Balance>,

	/// Initial claims for proof of existence
	/// Maps content to the account that owns the claim
	pub claims: BTreeMap<Content, AccountId>,

	/// Initial block number (defaults to 0)
	/// Allows starting from a specific block number if needed
	pub block_number: BlockNumber,

	/// Initial account nonces (defaults to empty)
	/// Maps account IDs to their starting nonce
	/// Typically empty for genesis, but can be pre-populated for testing
	pub nonces: BTreeMap<AccountId, Nonce>,
}

impl Default for GenesisConfig {
	fn default() -> Self {
		Self {
			balances: BTreeMap::new(),
			claims: BTreeMap::new(),
			block_number: 0,
			nonces: BTreeMap::new(),
		}
	}
}

impl GenesisConfig {
	/// Create a new empty genesis configuration
	pub fn new() -> Self {
		Self::default()
	}

	/// Create a new genesis configuration using the builder pattern
	pub fn builder() -> GenesisBuilder {
		GenesisBuilder::new()
	}

	/// Apply this genesis configuration to a runtime
	///
	/// This function initializes all pallets with the genesis state.
	/// It should be called immediately after creating a new runtime.
	pub fn apply_to(self, runtime: &mut crate::Runtime) {
		// Set the initial block number
		runtime.system.block_number = self.block_number;

		// Set initial nonces
		for (account_id, nonce) in self.nonces {
			runtime.system.nonce.insert(account_id, nonce);
		}

		// Set initial balances
		for (account_id, balance) in self.balances {
			runtime.balances.balances.insert(account_id, balance);
		}

		// Set initial claims
		for (content, account_id) in self.claims {
			runtime.proof_of_existence.claims.insert(content, account_id);
		}
	}
}

/// Builder for constructing a `GenesisConfig`
///
/// Provides a fluent API for setting up the initial blockchain state.
///
/// # Example
///
/// ```
/// use crate::genesis::GenesisConfig;
///
/// let genesis = GenesisConfig::builder()
///     .add_balance(alice_account, 1000)
///     .add_balance(bob_account, 500)
///     .add_claim("my_document", alice_account)
///     .build();
/// ```
#[derive(Debug, Clone, Default)]
pub struct GenesisBuilder {
	balances: BTreeMap<AccountId, Balance>,
	claims: BTreeMap<Content, AccountId>,
	block_number: BlockNumber,
	nonces: BTreeMap<AccountId, Nonce>,
}

impl GenesisBuilder {
	/// Create a new builder with default values
	pub fn new() -> Self {
		Self::default()
	}

	/// Add an initial balance for an account
	///
	/// # Arguments
	/// * `account_id` - The account ID to set the balance for
	/// * `balance` - The initial balance amount
	pub fn add_balance(mut self, account_id: AccountId, balance: Balance) -> Self {
		self.balances.insert(account_id, balance);
		self
	}

	/// Add multiple initial balances at once
	///
	/// # Arguments
	/// * `balances` - An iterator of (account_id, balance) pairs
	pub fn add_balances<I>(mut self, balances: I) -> Self
	where
		I: IntoIterator<Item = (AccountId, Balance)>,
	{
		self.balances.extend(balances);
		self
	}

	/// Add an initial claim for proof of existence
	///
	/// # Arguments
	/// * `content` - The content being claimed
	/// * `account_id` - The account that owns the claim
	pub fn add_claim(mut self, content: Content, account_id: AccountId) -> Self {
		self.claims.insert(content, account_id);
		self
	}

	/// Add multiple initial claims at once
	///
	/// # Arguments
	/// * `claims` - An iterator of (content, account_id) pairs
	pub fn add_claims<I>(mut self, claims: I) -> Self
	where
		I: IntoIterator<Item = (Content, AccountId)>,
	{
		self.claims.extend(claims);
		self
	}

	/// Set the initial block number
	///
	/// # Arguments
	/// * `block_number` - The starting block number
	pub fn with_block_number(mut self, block_number: BlockNumber) -> Self {
		self.block_number = block_number;
		self
	}

	/// Add an initial nonce for an account
	///
	/// # Arguments
	/// * `account_id` - The account ID to set the nonce for
	/// * `nonce` - The initial nonce value
	///
	/// # Note
	/// Nonces are typically empty at genesis, but this can be useful for testing.
	pub fn add_nonce(mut self, account_id: AccountId, nonce: Nonce) -> Self {
		self.nonces.insert(account_id, nonce);
		self
	}

	/// Build the final `GenesisConfig`
	pub fn build(self) -> GenesisConfig {
		GenesisConfig {
			balances: self.balances,
			claims: self.claims,
			block_number: self.block_number,
			nonces: self.nonces,
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_default_genesis() {
		let genesis = GenesisConfig::default();
		assert!(genesis.balances.is_empty());
		assert!(genesis.claims.is_empty());
		assert_eq!(genesis.block_number, 0);
		assert!(genesis.nonces.is_empty());
	}

	#[test]
	fn test_builder_add_balance() {
		let account: AccountId = [1u8; 32];
		let genesis = GenesisConfig::builder().add_balance(account, 100).build();
		assert_eq!(genesis.balances.get(&account), Some(&100));
	}

	#[test]
	fn test_builder_add_balances() {
		let account1: AccountId = [1u8; 32];
		let account2: AccountId = [2u8; 32];
		let balances = vec![(account1, 100), (account2, 200)];

		let genesis = GenesisConfig::builder().add_balances(balances).build();
		assert_eq!(genesis.balances.len(), 2);
		assert_eq!(genesis.balances.get(&account1), Some(&100));
		assert_eq!(genesis.balances.get(&account2), Some(&200));
	}

	#[test]
	fn test_builder_add_claim() {
		let account: AccountId = [1u8; 32];
		let genesis = GenesisConfig::builder().add_claim("test_content", account).build();
		assert_eq!(genesis.claims.get("test_content"), Some(&account));
	}

	#[test]
	fn test_builder_with_block_number() {
		let genesis = GenesisConfig::builder().with_block_number(5).build();
		assert_eq!(genesis.block_number, 5);
	}

	#[test]
	fn test_builder_comprehensive() {
		let alice: AccountId = [1u8; 32];
		let bob: AccountId = [2u8; 32];

		let genesis = GenesisConfig::builder()
			.add_balance(alice, 1000)
			.add_balance(bob, 500)
			.add_claim("doc1", alice)
			.add_claim("doc2", bob)
			.with_block_number(1)
			.add_nonce(alice, 0)
			.build();

		assert_eq!(genesis.balances.len(), 2);
		assert_eq!(genesis.balances.get(&alice), Some(&1000));
		assert_eq!(genesis.balances.get(&bob), Some(&500));
		assert_eq!(genesis.claims.len(), 2);
		assert_eq!(genesis.block_number, 1);
		assert_eq!(genesis.nonces.get(&alice), Some(&0));
	}
}
