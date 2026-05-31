// Block struct representing a full block in the blockchain
// Generic over Header and Extrinsic types
pub struct Block<Header, Extrinsic> {
	// The block header containing metadata like block number
	pub header: Header,
	// The list of extrinsics (transactions) in this block
	pub extrinsics: Vec<Extrinsic>,
}

// Header struct representing block metadata
// Generic over BlockNumber type
pub struct Header<BlockNumber> {
	// The sequential number of this block in the chain
	pub block_number: BlockNumber,
}

// Extrinsic struct representing a single transaction/operation
// Generic over Caller and Call types
pub struct Extrinsic<Caller, Call> {
	// The account/origin that is calling this function (derived from public key)
	pub caller: Caller,
	// The actual function call being executed
	pub call: Call,
	// The Ed25519 signature authorizing this transaction
	pub signature: crate::crypto::SignatureWrapper,
	// The nonce for replay protection
	pub nonce: u32,
}

// Type alias for the result of a dispatch operation
// Returns Ok(()) on success, or a static error string on failure
pub type DispatchResult = Result<(), &'static str>;

// Dispatch trait that allows a pallet to execute calls
// This trait must be implemented by pallets that have callable functions
pub trait Dispatch {
	// Associated type for the caller (account ID)
	type Caller;
	// Associated type for the call (the function to execute)
	type Call;

	// Dispatch a call on behalf of a caller
	// Returns Ok(()) on success, or an error message string on failure
	fn dispatch(&mut self, caller: Self::Caller, call: Self::Call) -> DispatchResult;
}
