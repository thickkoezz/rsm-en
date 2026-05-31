//! Persistent storage module for blockchain state
//!
//! This module handles serialization and persistence of all blockchain state
//! using sled as an embedded key-value database.

// Import necessary items from the parent crate module
use crate::{
	Runtime,                                                  // Import the main Runtime struct
	event::{Event, EventRecord, Phase},                       // Import event-related types
	types::{AccountId, Balance, BlockNumber, Content, Nonce}, // Import core type aliases
};

// Import serde traits for serialization/deserialization
use serde::{Deserialize, Serialize}; // Enable serializing/deserializing structs
// Import standard library collections and path handling
use std::{collections::BTreeMap, path::Path}; // BTreeMap for ordered storage, Path for file paths

/// Serializable event that uses String instead of &'static str for Content
// Define an enum that can be serialized, holding event data with owned strings
#[derive(Serialize, Deserialize)] // Enable automatic serialization
enum SerializableEvent {
	BalanceTransfer(AccountId, AccountId, Balance), // Transfer event: from, to, amount
	ClaimCreated(AccountId, String),                /* Claim creation event: account, content
	                                                 * (String) */
	ClaimRevoked(AccountId, String), // Claim revocation event: account, content (String)
	FeePaid(AccountId, Balance),     // Fee payment event: payer, amount
	InsufficientFee(AccountId, Balance, Balance), // Insufficient fee event: payer, required, actual
}

/// Serializable event record
// Define a struct that wraps an event with its phase information
#[derive(Serialize, Deserialize)] // Enable automatic serialization
struct SerializableEventRecord {
	phase: Phase,             // The phase when the event occurred
	event: SerializableEvent, // The actual event data
}

// Implement conversion methods for SerializableEvent
impl SerializableEvent {
	/// Convert from a regular Event to SerializableEvent
	// Convert from a borrowed Event with lifetime to owned SerializableEvent
	fn from_event(event: &Event<AccountId, Balance, Content>) -> Self {
		match event {
			// Match on the event type
			Event::BalanceTransfer(from, to, amount) =>
			// Case: balance transfer
				SerializableEvent::BalanceTransfer(*from, *to, *amount), // Copy values into owned variant
			Event::ClaimCreated(account, content) =>
			// Case: claim created
				SerializableEvent::ClaimCreated(*account, content.to_string()), /* Copy account, convert */
			// content to String
			Event::ClaimRevoked(account, content) =>
			// Case: claim revoked
				SerializableEvent::ClaimRevoked(*account, content.to_string()), /* Copy account, convert */
			// content to String
			Event::FeePaid(payer, fee) => SerializableEvent::FeePaid(*payer, *fee), // Case: fee paid
			Event::InsufficientFee(payer, required, actual) =>
			// Case: insufficient fee
				SerializableEvent::InsufficientFee(*payer, *required, *actual), /* Copy all values
			                                                                          * into owned
			                                                                          * variant */
		} // End match
	}

	/// Convert from SerializableEvent to regular Event
	/// This leaks strings to convert them to &'static str
	// Convert back from owned to borrowed Event, leaking memory for strings
	fn to_event(&self) -> Event<AccountId, Balance, Content> {
		match self {
			// Match on self (the SerializableEvent)
			SerializableEvent::BalanceTransfer(from, to, amount) =>
			// Case: balance transfer
				Event::BalanceTransfer(*from, *to, *amount), // Copy values into Event variant
			SerializableEvent::ClaimCreated(account, content) => {
				// Case: claim created
				// Leak the string to get &'static str - safe because it lives for program duration
				let leaked: &'static str = Box::leak(content.clone().into_boxed_str()); // Convert String to leaked &'static str
				Event::ClaimCreated(*account, leaked) // Create Event with leaked string reference
			}, // End case
			SerializableEvent::ClaimRevoked(account, content) => {
				// Case: claim revoked
				let leaked: &'static str = Box::leak(content.clone().into_boxed_str()); // Convert String to leaked &'static str
				Event::ClaimRevoked(*account, leaked) // Create Event with leaked string reference
			}, // End case
			SerializableEvent::FeePaid(payer, fee) => Event::FeePaid(*payer, *fee), // Case: fee paid
			SerializableEvent::InsufficientFee(payer, required, actual) =>
			// Case: insufficient fee
				Event::InsufficientFee(*payer, *required, *actual), /* Copy all values
			                                                                          * into Event
			                                                                          * variant */
		} // End match
	}
}

/// Serializable representation of the blockchain state
// Define a struct that holds all blockchain state in a serializable format
#[derive(Serialize, Deserialize)] // Enable automatic serialization
struct PersistentState {
	/// Current block number
	block_number: BlockNumber, // The current block number in the chain
	/// Account nonces
	nonces: BTreeMap<String, Nonce>, // Map of hex-encoded account IDs to their nonces
	/// Account balances
	balances: BTreeMap<String, Balance>, // Map of hex-encoded account IDs to their balances
	/// Claims (proof of existence)
	claims: BTreeMap<String, AccountId>, // Map of content strings to owning account IDs
	/// Events indexed by (block_number, extrinsic_index)
	events: BTreeMap<(BlockNumber, u32), SerializableEventRecord>, /* Map of (block, extrinsic) to
	                                                                * event records */
	/// Total fees collected
	total_fees: Balance, // Total amount of fees collected by the system
}

/// Storage backend using sled database
// Public struct that wraps the sled database for blockchain state persistence
pub struct Storage {
	db: sled::Db, // The sled database instance
}

// Implement methods for the Storage struct
impl Storage {
	/// Open or create a new database at the specified path
	// Open or create a sled database at the given path
	pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
		let db = sled::open(path)?; // Open database at specified path
		Ok(Self { db }) // Return Storage wrapping the database
	}

	/// Create a new in-memory database (useful for testing)
	// Create a temporary in-memory database for testing purposes
	pub fn in_memory() -> Result<Self, Box<dyn std::error::Error>> {
		let db = sled::Config::new().temporary(true).open()?; // Create temporary in-memory database
		Ok(Self { db }) // Return Storage wrapping the database
	}

	/// Save the complete runtime state to disk
	// Serialize and persist the entire blockchain state to the database
	pub fn save_state(&self, runtime: &Runtime) -> Result<(), Box<dyn std::error::Error>> {
		// Build the persistent state from the runtime
		let state = self.extract_state(runtime); // Extract all state from runtime into serializable struct

		// Serialize the state
		let serialized = bincode::serialize(&state)?; // Serialize the state to binary format

		// Store in the database
		self.db.insert("state", serialized)?; // Insert serialized data into database

		// Flush to ensure data is written to disk
		self.db.flush()?; // Flush database to ensure data is persisted

		Ok(()) // Return success
	}

	/// Load the runtime state from disk, or create a new runtime if no state exists
	///
	/// # Arguments
	/// * `genesis` - Optional genesis configuration to apply when creating a new runtime
	///
	/// # Returns
	/// The loaded runtime or a new runtime initialized with the genesis config
	pub fn load_state_or_create(
		&self,
		genesis: Option<crate::genesis::GenesisConfig>,
	) -> Result<Runtime, Box<dyn std::error::Error>> {
		match self.db.get("state")? {
			// Try to get state from database
			Some(data) => {
				// Case: state exists
				// Deserialize existing state
				let state: PersistentState = bincode::deserialize(&data)?; // Deserialize binary data to PersistentState
				Ok(self.build_runtime(state)) // Build and return runtime from state
			}, // End Some case
			None => {
				// Case: no state exists
				// No existing state, create new runtime with genesis config if provided
				let mut runtime = Runtime::new(); // Create new empty runtime
				if let Some(genesis_config) = genesis {
					// Apply genesis configuration if provided
					genesis_config.apply_to(&mut runtime);
				}
				Ok(runtime) // Return the configured runtime
			}, // End None case
		} // End match
	}

	/// Extract the persistent state from the runtime
	// Convert the runtime state into a serializable PersistentState struct
	fn extract_state(&self, runtime: &Runtime) -> PersistentState {
		// Extract system state
		let block_number = runtime.system.block_number; // Get current block number from system

		// Extract nonces - convert AccountId keys to Strings for serialization
		let nonces: BTreeMap<String, Nonce> =                           // Create map of hex-encoded account IDs to nonces
			runtime.system.nonce.iter().map(|(k, v)| (hex::encode(k), *v)).collect(); // Encode each AccountId to hex string

		// Extract balances - convert AccountId keys to Strings
		let balances: BTreeMap<String, Balance> =                        // Create map of hex-encoded account IDs to balances
			runtime.balances.balances.iter().map(|(k, v)| (hex::encode(k), *v)).collect(); // Encode each AccountId to hex string

		// Extract claims - convert Content keys to Strings
		let claims: BTreeMap<String, AccountId> = runtime // Create map of content strings to account IDs
			.proof_of_existence // Access proof_of_existence pallet
			.claims // Access claims map
			.iter() // Create iterator over claims
			.map(|(k, v)| (k.to_string(), *v)) // Convert each &'static str key to String
			.collect(); // Collect into BTreeMap

		// Extract events - convert to serializable format
		let events: BTreeMap<(BlockNumber, u32), SerializableEventRecord> =
			runtime // Create map of (block, extrinsic) to events
				.events // Access events pallet
				.events // Access events map
				.iter() // Create iterator over events
				.map(|((block_num, ext_idx), record)| {
					// Transform each event
					(
						// Return tuple:
						(*block_num, *ext_idx), // Copy block number and extrinsic index as key
						SerializableEventRecord {
							// Create serializable event record
							phase: record.phase.clone(), // Clone the phase
							event: SerializableEvent::from_event(&record.event), // Convert event to serializable format
						}, // End SerializableEventRecord
					) // End tuple
				}) // End map closure
				.collect(); // Collect into BTreeMap

		// Extract total fees
		let total_fees = runtime.fees.total_fees_collected; // Get total fees collected from fees pallet

		PersistentState { block_number, nonces, balances, claims, events, total_fees } // Construct and return PersistentState
	}

	/// Build a runtime from the persistent state
	// Reconstruct a Runtime from a deserialized PersistentState
	fn build_runtime(&self, state: PersistentState) -> Runtime {
		let mut runtime = Runtime::new(); // Create a new empty runtime

		// Restore block number
		runtime.system.block_number = state.block_number; // Set block number from state

		// Restore nonces - convert hex strings back to AccountId
		for (hex_key, nonce) in state.nonces {
			// Iterate over nonce entries
			if let Ok(account_id) = hex::decode(&hex_key) {
				// Decode hex string to bytes
				if account_id.len() == 32 {
					// Check if account ID is correct length
					let mut bytes = [0u8; 32]; // Create fixed-size array
					bytes.copy_from_slice(&account_id); // Copy decoded bytes into array
					runtime.system.nonce.insert(bytes, nonce); // Insert into system nonce map
				} // End if
			} // End if let
		} // End for

		// Restore balances - convert hex strings back to AccountId
		for (hex_key, balance) in state.balances {
			// Iterate over balance entries
			if let Ok(account_id) = hex::decode(&hex_key) {
				// Decode hex string to bytes
				if account_id.len() == 32 {
					// Check if account ID is correct length
					let mut bytes = [0u8; 32]; // Create fixed-size array
					bytes.copy_from_slice(&account_id); // Copy decoded bytes into array
					runtime.balances.balances.insert(bytes, balance); // Insert into balances map
				} // End if
			} // End if let
		} // End for

		// Restore claims - Content is &'static str, so we need to leak the strings
		// This is safe because the strings come from persistent storage and will live for the program
		// duration
		for (content_str, account_id) in state.claims {
			// Iterate over claim entries
			// Leak the string to get &'static str
			let leaked: &'static str = Box::leak(content_str.into_boxed_str()); // Leak String to get &'static str
			runtime.proof_of_existence.claims.insert(leaked, account_id); // Insert into claims map
		} // End for

		// Restore events
		for ((block_num, ext_idx), record) in state.events {
			// Iterate over event entries
			runtime.events.events.insert(
				// Insert into events map
				(block_num, ext_idx), // Use (block, extrinsic) as key
				EventRecord { phase: record.phase, event: record.event.to_event() }, // Convert to EventRecord
			); // End insert
		} // End for

		// Restore total fees
		runtime.fees.total_fees_collected = state.total_fees; // Set total fees from state

		runtime // Return the constructed runtime
	}

	/// Check if persistent state exists
	// Check whether the database contains any saved state
	pub fn has_state(&self) -> bool {
		self.db.contains_key("state").unwrap_or(false) // Check if "state" key exists, default to false on error
	}

	/// Clear all state (useful for resetting the blockchain)
	// Remove all persistent state from the database
	pub fn clear(&self) -> Result<(), Box<dyn std::error::Error>> {
		self.db.remove("state")?; // Remove the "state" entry from database
		self.db.flush()?; // Flush to ensure deletion is persisted
		Ok(()) // Return success
	}
}

// Test module for storage functionality
#[cfg(test)] // Only compile when running tests
mod tests {
	use super::*; // Import all items from parent module

	#[test] // Test: in-memory storage creation
	fn test_in_memory_storage() {
		let storage = Storage::in_memory().unwrap(); // Create in-memory storage
		assert!(!storage.has_state()); // Verify no state exists initially
	}

	#[test] // Test: save and load state
	fn test_save_and_load() {
		let storage = Storage::in_memory().unwrap(); // Create in-memory storage

		// Create a runtime and modify it
		let runtime = Runtime::new(); // Create new runtime instance

		// Save the state
		storage.save_state(&runtime).unwrap(); // Save runtime state to storage

		// Verify state exists
		assert!(storage.has_state()); // Verify state was saved

		// Load the state
		let _loaded_runtime = storage.load_state_or_create(None).unwrap(); // Load state from storage
	}

	#[test] // Test: load without existing state
	fn test_load_without_state() {
		let storage = Storage::in_memory().unwrap(); // Create in-memory storage

		// Load without any existing state should create new runtime
		let runtime = storage.load_state_or_create(None).unwrap(); // Load state (should create new)
		assert_eq!(runtime.system.block_number(), 0); // Verify block number is 0 (new runtime)
	}

	#[test] // Test: clear state
	fn test_clear() {
		let storage = Storage::in_memory().unwrap(); // Create in-memory storage
		let runtime = Runtime::new(); // Create new runtime instance

		// Save state
		storage.save_state(&runtime).unwrap(); // Save runtime state to storage
		assert!(storage.has_state()); // Verify state was saved

		// Clear state
		storage.clear().unwrap(); // Clear the state from storage
		assert!(!storage.has_state()); // Verify state was cleared
	}

	#[test] // Test: event serialization conversion
	fn test_serializable_event_conversion() {
		let event = Event::BalanceTransfer([1u8; 32], [2u8; 32], 100); // Create a balance transfer event
		let serializable = SerializableEvent::from_event(&event); // Convert to serializable format
		let restored = serializable.to_event(); // Convert back to regular event

		match restored {
			// Match on the restored event
			Event::BalanceTransfer(from, to, amount) => {
				// Case: balance transfer
				assert_eq!(from, [1u8; 32]); // Verify from account matches
				assert_eq!(to, [2u8; 32]); // Verify to account matches
				assert_eq!(amount, 100); // Verify amount matches
			}, // End case
			_ => panic!("Unexpected event type"), // Panic for any other event type
		} // End match
	}
}
