// Import the event record and phase types for event metadata
use crate::event::{EventRecord, Phase};
// Import standard library collections
use std::collections::BTreeMap;

// Configuration trait for the Events pallet
// Must also implement the system::Config trait
pub trait Config: crate::system::Config {
	// Event type must support cloning and debug formatting
	type Event: Clone + core::fmt::Debug;
}

// Derive Debug and Clone traits for the Pallet struct
#[derive(Debug, Clone)]
// The Events Pallet struct that manages event storage
// Generic over T which implements Config
pub struct Pallet<T: Config> {
	// Storage mapping from (block_number, extrinsic_index) to EventRecord
	// This allows querying events by block and optionally by extrinsic
	pub events: BTreeMap<(T::BlockNumber, u32), EventRecord<T::Event>>,
}

// Implementation of Events Pallet methods
impl<T: Config> Pallet<T> {
	// Create a new Events Pallet with initial state
	pub fn new() -> Self {
		// Return a new Pallet with an empty events map
		Self { events: BTreeMap::new() }
	}

	// Deposit an event into the event log with explicit block number
	// This is called when a pallet wants to emit an event
	pub fn deposit_event(&mut self, block_number: T::BlockNumber, phase: Phase, event: T::Event) {
		// Determine the extrinsic index based on the phase
		let extrinsic_index = match phase {
			// Phase::Initialization uses index 0 (before any extrinsic)
			Phase::Initialization => 0,
			// Phase::ApplyExtrinsic contains the actual extrinsic index
			Phase::ApplyExtrinsic(idx) => idx,
			// Phase::Finalization uses max u32 value (after all extrinsics)
			Phase::Finalization => u32::MAX,
		};

		// Create an event record with the phase and event data
		let record = EventRecord { phase: phase.clone(), event };
		// Insert the event record into storage keyed by block number and extrinsic index
		self.events.insert((block_number, extrinsic_index), record);
	}

	// Get all events that occurred at a specific block
	// Returns a vector of EventRecord containing all events from that block
	pub fn events_at_block(&self, block: T::BlockNumber) -> Vec<EventRecord<T::Event>> {
		// Iterate through all events, filter by block number, and collect results
		self
			.events
			.iter()
			.filter(|((block_num, _), _)| *block_num == block)
			.map(|(_, event)| event.clone())
			.collect()
	}

	// Get a specific event for a particular extrinsic in a block
	// Returns Option<EventRecord> containing the event if it exists
	pub fn event_for_extrinsic(
		&self,
		// The block number to query
		block: T::BlockNumber,
		// The extrinsic index within the block
		extrinsic: u32,
	) -> Option<EventRecord<T::Event>> {
		// Look up the event by block number and extrinsic index
		self.events.get(&(block, extrinsic)).cloned()
	}

	// Clear all events from storage
	// This is typically called at the start of each new block
	pub fn clear_events(&mut self) {
		// Remove all events from the map
		self.events.clear();
	}
}

// Test module for Events Pallet
#[cfg(test)]
mod test {
	// Test configuration struct for unit tests
	struct TestConfig;

	// Implement system::Config trait for TestConfig
	impl crate::system::Config for TestConfig {
		// Use String as AccountId for testing
		type AccountId = String;
		// Use u32 as BlockNumber for testing
		type BlockNumber = u32;
		// Use u32 as Nonce for testing
		type Nonce = u32;
	}

	// Implement Events::Config trait for TestConfig
	impl super::Config for TestConfig {
		// Use the Event enum from the event module with String for AccountId, u128 for Balance, and
		// &'static str for Content
		type Event = crate::event::Event<String, u128, &'static str>;
	}

	// Test that events can be deposited
	#[test]
	fn deposit_event() {
		// Create a mutable Events Pallet instance for testing
		let mut events: super::Pallet<TestConfig> = super::Pallet::new();

		// Create a test balance transfer event
		let event = crate::event::Event::BalanceTransfer("alice".to_string(), "bob".to_string(), 100);

		// Deposit the event in block 0 during extrinsic 0
		events.deposit_event(0, crate::event::Phase::ApplyExtrinsic(0), event);

		// Assert that one event is now stored
		assert_eq!(events.events.len(), 1);
	}

	// Test that events can be retrieved by block
	#[test]
	fn events_at_block() {
		// Create a mutable Events Pallet instance for testing
		let mut events: super::Pallet<TestConfig> = super::Pallet::new();

		// Create two test balance transfer events
		let event1 = crate::event::Event::BalanceTransfer("alice".to_string(), "bob".to_string(), 100);
		let event2 = crate::event::Event::BalanceTransfer("bob".to_string(), "charlie".to_string(), 50);

		// Deposit both events in block 1
		events.deposit_event(1, crate::event::Phase::ApplyExtrinsic(0), event1);
		events.deposit_event(1, crate::event::Phase::ApplyExtrinsic(1), event2);

		// Get all events from block 1
		let block_events = events.events_at_block(1);
		// Assert that two events were found
		assert_eq!(block_events.len(), 2);
	}

	// Test that events can be cleared
	#[test]
	fn clear_events() {
		// Create a mutable Events Pallet instance for testing
		let mut events: super::Pallet<TestConfig> = super::Pallet::new();

		// Create a test balance transfer event
		let event = crate::event::Event::BalanceTransfer("alice".to_string(), "bob".to_string(), 100);

		// Deposit the event
		events.deposit_event(0, crate::event::Phase::ApplyExtrinsic(0), event);
		// Assert that one event is stored
		assert_eq!(events.events.len(), 1);

		// Clear all events
		events.clear_events();
		// Assert that no events remain
		assert_eq!(events.events.len(), 0);
	}
}
