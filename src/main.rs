// Import the Dispatch trait from the support module for handling extrinsic dispatching
use crate::support::Dispatch;

// Declare the balances pallet module
mod balances;
// Declare the event module for event types
mod event;
// Declare the events pallet module
mod events;
// Declare the proof_of_existence pallet module
mod proof_of_existence;
// Declare the support module for common types and traits
mod support;
// Declare the crypto module for cryptographic primitives
mod crypto;
// Declare the transaction module for building signed transactions
mod transaction;
// Declare the system pallet module
mod system;

// Define a module for type aliases to make the code more readable and maintainable
mod types {
	// Import the support module to access its types
	use crate::support;

	// Type alias for AccountId - now a 32-byte public key
	pub type AccountId = [u8; 32];
	// Type alias for Balance - represents the token balance with u128 for large values
	pub type Balance = u128;
	// Type alias for BlockNumber - represents the sequential number of a block
	pub type BlockNumber = u32;
	// Type alias for Nonce - represents the transaction count for an account
	pub type Nonce = u32;
	// Type alias for Extrinsic - represents a transaction/call with caller and runtime call data
	pub type Extrinsic = support::Extrinsic<AccountId, crate::RuntimeCall>;
	// Type alias for Header - represents the block header containing block number
	pub type Header = support::Header<BlockNumber>;
	// Type alias for Block - represents a full block with header and extrinsics
	pub type Block = support::Block<Header, Extrinsic>;
	// Type alias for Content - represents static string content for proofs
	pub type Content = &'static str;
}

// Derive Debug and Clone traits for the Runtime struct
// Debug allows printing the struct for debugging
// Clone allows creating copies of the runtime
#[derive(Debug, Clone)]
// Use the runtime procedural macro to automatically generate dispatch code
#[macros::runtime]
// The main Runtime struct that contains all pallets (modules) of the blockchain
pub struct Runtime {
	// System pallet: manages block numbers and nonces (transaction counts)
	system: system::Pallet<Runtime>,
	// Events pallet: manages and stores events emitted during block execution
	events: events::Pallet<Runtime>,
	// Balances pallet: manages account balances and transfers
	balances: balances::Pallet<Runtime>,
	// Proof of Existence pallet: manages claims/proofs of data existence
	proof_of_existence: proof_of_existence::Pallet<Runtime>,
}

// Implement the system::Config trait for Runtime to configure the system pallet
impl system::Config for Runtime {
	// Specify that AccountId is our 32-byte public key array type
	type AccountId = types::AccountId;
	// Specify that BlockNumber is our u32 type
	type BlockNumber = types::BlockNumber;
	// Specify that Nonce is our u32 type
	type Nonce = types::Nonce;
}

// Implement the balances::Config trait for Runtime to configure the balances pallet
impl balances::Config for Runtime {
	// Specify that Balance is our u128 type
	type Balance = types::Balance;
}

// Implement the proof_of_existence::Config trait for Runtime to configure the PoE pallet
impl proof_of_existence::Config for Runtime {
	// Specify that Content is our static string type
	type Content = types::Content;
}

// Implement the events::Config trait for Runtime to configure the events pallet
impl events::Config for Runtime {
	// Specify that Event is our Event enum type with AccountId, Balance, and Content type parameters
	type Event = event::Event<types::AccountId, types::Balance, types::Content>;
}

// The main entry point of the program
fn main() {
	// Create a new Runtime instance by calling the generated new() function
	let mut runtime = Runtime::new();

	// Generate keypairs for Alice, Bob, and Charlie
	let alice_keypair = crate::crypto::KeypairWrapper::generate();
	let bob_keypair = crate::crypto::KeypairWrapper::generate();
	let charlie_keypair = crate::crypto::KeypairWrapper::generate();

	// Get the account IDs (public keys) for each user
	let alice_account: [u8; 32] = alice_keypair.public().clone().into();
	let bob_account: [u8; 32] = bob_keypair.public().clone().into();
	let charlie_account: [u8; 32] = charlie_keypair.public().clone().into();

	// Set Alice's initial balance to 100 tokens using the balances pallet
	runtime.balances.set_balance(alice_account, 100);

	// Create Block 1 with two transfer extrinsics
	let block_1 = types::Block {
		// Set the block number to 1
		header: support::Header { block_number: 1 },
		// Define the extrinsics (transactions) in this block
		extrinsics: vec![
			// First extrinsic: Alice transfers 30 tokens to Bob
			transaction::TransactionBuilder::signed_extrinsic(
				&alice_keypair,
				RuntimeCall::balances(balances::Call::transfer { to: bob_account, amount: 30 }),
				0, // nonce for Alice's first transaction
			),
			// Second extrinsic: Alice transfers 20 tokens to Charlie
			transaction::TransactionBuilder::signed_extrinsic(
				&alice_keypair,
				RuntimeCall::balances(balances::Call::transfer { to: charlie_account, amount: 20 }),
				1, // nonce for Alice's second transaction
			),
		],
	};

	// Execute block 1, expecting successful execution (no errors)
	runtime.execute_block(block_1).expect("wrong block execution");

	// Query events from block 1 immediately after execution
	// Print a header for the events section
	println!("\n=== Events in Block 1 ===");
	// Iterate through and print all events that occurred in block 1
	for event in runtime.events.events_at_block(1) {
		// Print each event with debug formatting
		println!("  {:?}", event);
	}

	// Create Block 2 with two claim creation extrinsics
	let block_2 = types::Block {
		// Set the block number to 2
		header: support::Header { block_number: 2 },
		// Define the extrinsics (transactions) in this block
		extrinsics: vec![
			// First extrinsic: Alice creates a claim for "my_document"
			transaction::TransactionBuilder::signed_extrinsic(
				&alice_keypair,
				RuntimeCall::proof_of_existence(proof_of_existence::Call::create_claim {
					// The content/claim being proven
					claim: "my_document",
				}),
				2, // nonce for Alice's third transaction
			),
			// Second extrinsic: Bob creates a claim for "bob's_document"
			transaction::TransactionBuilder::signed_extrinsic(
				&bob_keypair,
				RuntimeCall::proof_of_existence(proof_of_existence::Call::create_claim {
					// The content/claim being proven
					claim: "bob's_document",
				}),
				0, // nonce for Bob's first transaction
			),
		],
	};

	// Execute block 2, expecting successful execution (no errors)
	runtime.execute_block(block_2).expect("wrong block execution");

	// Print the entire runtime state for debugging/inspection
	println!("{:#?}", runtime);

	// Query and display events from block 1 again
	// Print a header for the events section
	println!("\n=== Events in Block 1 ===");
	// Iterate through and print all events that occurred in block 1
	for event in runtime.events.events_at_block(1) {
		// Print each event with debug formatting
		println!("  {:?}", event);
	}

	// Query and display events from block 2
	// Print a header for the events section
	println!("\n=== Events in Block 2 ===");
	// Iterate through and print all events that occurred in block 2
	for event in runtime.events.events_at_block(2) {
		// Print each event with debug formatting
		println!("  {:?}", event);
	}

	// Demonstrate error cases

	// 1. Test replay attack prevention - try to replay Alice's first transaction
	println!("\n=== Testing Replay Attack Prevention ===");
	let replay_block = types::Block {
		header: support::Header { block_number: 3 },
		extrinsics: vec![
			// Try to replay Alice's first transaction with nonce 0
			transaction::TransactionBuilder::signed_extrinsic(
				&alice_keypair,
				RuntimeCall::balances(balances::Call::transfer { to: bob_account, amount: 30 }),
				0, // Wrong nonce - should be 3
			),
		],
	};

	match runtime.execute_block(replay_block) {
		Ok(_) => println!("ERROR: Replay attack succeeded!"),
		Err(e) => println!("Replay attack prevented: {}", e),
	}

	// 2. Test invalid signature prevention
	// Create a valid signed extrinsic with Alice, then tamper with the caller to make signature
	// invalid
	println!("\n=== Testing Invalid Signature Prevention ===");

	// Create a valid signed extrinsic with Alice's keypair
	let mut tampered_extrinsic = transaction::TransactionBuilder::signed_extrinsic(
		&alice_keypair,
		RuntimeCall::balances(balances::Call::transfer { to: bob_account, amount: 10 }),
		3, // Alice's correct next nonce
	);

	// Tamper with the caller and nonce to bypass nonce check but break signature verification
	tampered_extrinsic.caller = charlie_account; // Change caller to Charlie's account
	tampered_extrinsic.nonce = 0; // Use Charlie's correct nonce (0) to pass nonce check

	// The signature was signed by Alice for her account and nonce 3
	// Now we're trying to verify it against Charlie's public key with nonce 0
	// This should cause signature verification to fail
	let fake_signature_block = types::Block {
		header: support::Header { block_number: 4 },
		extrinsics: vec![tampered_extrinsic],
	};

	// This should fail due to signature verification mismatch
	match runtime.execute_block(fake_signature_block) {
		Ok(_) => println!("ERROR: Transaction with invalid signature succeeded!"),
		Err(e) => println!("Invalid signature prevented: {}", e),
	}
}
