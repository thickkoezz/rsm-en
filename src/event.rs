// Import the Debug trait from core for formatting
use core::fmt::Debug;
use serde::{Deserialize, Serialize};

// EventRecord struct that wraps an event with metadata about when it occurred
// Generic over the Event type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventRecord<Event> {
	// The phase indicates when the event occurred (initialization, extrinsic, or finalization)
	pub phase: Phase,
	// The actual event data
	pub event: Event,
}

// Phase enum indicating when during block execution the event occurred
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Phase {
	// Phase during block initialization (before any extrinsics are executed)
	Initialization,
	// Phase during extrinsic execution (contains the index of the extrinsic)
	ApplyExtrinsic(u32), // extrinsic index
	// Phase during block finalization (after all extrinsics are executed)
	Finalization,
}

// Event enum containing all possible events from all pallets
// This is the aggregate event type that the runtime uses
// Generic over AccountId, Balance, and Content types for flexibility
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Event<AccountId, Balance, Content> {
	// Event emitted when a balance transfer occurs
	// Contains: from_account, to_account, amount
	BalanceTransfer(AccountId, AccountId, Balance),
	// Event emitted when a claim is created
	// Contains: account_who_created, claim_content
	ClaimCreated(AccountId, Content),
	// Event emitted when a claim is revoked
	// Contains: account_who_revoked, claim_content
	ClaimRevoked(AccountId, Content),
	// Event emitted when a transaction fee is paid
	// Contains: payer_account, fee_amount
	FeePaid(AccountId, Balance),
	// Event emitted when fee payment fails
	// Contains: payer_account, required_fee, actual_balance
	InsufficientFee(AccountId, Balance, Balance),
}
