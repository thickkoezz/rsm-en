// Import the RuntimeDef struct from the parse module
use super::parse::RuntimeDef;
// Import the quote macro for code generation
use quote::quote;

// See the `fn runtime` docs at the `lib.rs` of this crate for a high level definition.
// This function generates the runtime initialization and dispatch code
pub fn expand_runtime(def: RuntimeDef) -> proc_macro2::TokenStream {
	// Destructure the RuntimeDef to get its components
	let RuntimeDef { runtime_struct, pallets } = def;

	// Filter out pallets that don't have callable functions (like events)
	// We assume pallets with callable functions have a Call enum
	let callable_pallets: Vec<_> = pallets.iter().filter(|(name, _)| {
		// Skip the events pallet as it doesn't have callable functions
		name.to_string() != "events"
	}).collect();

	// This is a vector of all the pallet names, not including system and non-callable pallets.
	let pallet_names = callable_pallets.iter().map(|(name, _)| name.clone()).collect::<Vec<_>>();
	// This is a vector of all the pallet types, not including system and non-callable pallets.
	let _pallet_types = callable_pallets.iter().map(|(_, type_)| type_.clone()).collect::<Vec<_>>();

	// All pallets (including non-callable ones) for initialization
	let all_pallet_names = pallets.iter().map(|(name, _)| name.clone()).collect::<Vec<_>>();
	let all_pallet_types = pallets.iter().map(|(_, type_)| type_.clone()).collect::<Vec<_>>();

	// This quote block implements functions on the `Runtime` struct.
	let runtime_impl = quote! {
		// Implement methods on the Runtime struct
		impl #runtime_struct {
			// Create a new instance of the main Runtime, by creating a new instance of each pallet.
			fn new() -> Self {
				Self {
					// Since system is not included in the list of pallets, we manually add it here.
					system: <system::Pallet::<Self>>::new(),
					// Initialize all other pallets
					#(
						#all_pallet_names: <#all_pallet_types>::new()
					),*
				}
			}

			// Execute a block of extrinsics. Increments the block number.
			fn execute_block(&mut self, block: types::Block) -> crate::support::DispatchResult {
				// Clear events from previous block
				self.events.clear_events();

				// Increment the block number at the start of block execution
				self.system.inc_block_number();
				// Verify that the block number matches what we expect
				if block.header.block_number != self.system.block_number() {
					// Block number mismatch - return error
					return Err(&"block number does not match what is expected")
				}
				// Iterate over all extrinsics in the block
				for (i, support::Extrinsic { caller, call, signature, nonce }) in block.extrinsics.into_iter().enumerate() {
					// Verify the nonce matches the expected value BEFORE processing
					self.system.verify_nonce(&caller, nonce).map_err(|e| {
						eprintln!(
							"Nonce Verification Error\n\tBlock Number: {}\n\tExtrinsic Number: {}\n\tError: {}",
							block.header.block_number, i, e
						);
						&"Nonce verification failed"[..]
					})?;

					// Verify the signature is valid
					// Reconstruct the payload that was signed
					// We encode the full RuntimeCall, not just the inner call
					let call_bytes = serde_json::to_vec(&call).expect("Failed to encode call");
					let payload = crate::crypto::SignedPayload::new(call_bytes, nonce);
					let payload_hash = payload.hash();

					// Convert caller to public key wrapper for verification
					let public_key = crate::crypto::PublicKeyWrapper::try_from(caller)
						.map_err(|_| "Invalid caller public key")?;

					// Verify the signature
					crate::crypto::verify(&public_key, &signature, &payload_hash).map_err(|e| {
						eprintln!(
							"Signature Verification Error\n\tBlock Number: {}\n\tExtrinsic Number: {}\n\tError: {}",
							block.header.block_number, i, e
						);
						"Invalid signature"
					})?;

					// Increment the nonce for the caller (account)
					self.system.inc_nonce(&caller);

					// Store the call to emit events after successful execution
					// Clone the call for use in event emission
					let call_clone = match &call {
						// Clone balances calls
						crate::RuntimeCall::balances(inner) => {
							crate::RuntimeCall::balances(inner.clone())
						},
						// Clone proof_of_existence calls
						crate::RuntimeCall::proof_of_existence(inner) => {
							crate::RuntimeCall::proof_of_existence(inner.clone())
						},
					};

					// Dispatch the call and handle any errors
					let res = self.dispatch(caller.clone(), call);

					// Emit events on successful execution
					if res.is_ok() {
						// Create the phase for this extrinsic
						let phase = crate::event::Phase::ApplyExtrinsic(i as u32);
						// Match on the call to emit the appropriate event
						match call_clone {
							// Handle balance transfer events
							crate::RuntimeCall::balances(balances::Call::transfer { to, amount }) => {
								self.events.deposit_event(
									block.header.block_number,
									phase,
									crate::event::Event::BalanceTransfer(caller, to, amount),
								);
							},
							// Handle claim creation events
							crate::RuntimeCall::proof_of_existence(proof_of_existence::Call::create_claim { claim }) => {
								self.events.deposit_event(
									block.header.block_number,
									phase,
									crate::event::Event::ClaimCreated(caller, claim),
								);
							},
							// Handle claim revocation events
							crate::RuntimeCall::proof_of_existence(proof_of_existence::Call::revoke_claim { claim }) => {
								self.events.deposit_event(
									block.header.block_number,
									phase,
									crate::event::Event::ClaimRevoked(caller, claim),
								);
							},
							// Ignore other calls (no events to emit)
							_ => {},
						}
					}

					// Propagate any dispatch errors
					res.map_err(|e| {
						eprintln!(
							"Extrinsic Error\n\tBlock Number: {}\n\tExtrinsic Number: {}\n\tError: {}",
							block.header.block_number, i, e
						);
						e
					})?;
				}
				// Return success after all extrinsics are processed
				Ok(())
			}
		}
	};

	// This quote block implements the `RuntimeCall` enum and implements the `Dispatch` trait.
	let dispatch_impl = quote! {
		// These are all the calls which are exposed to the world.
		// Note that it is just an accumulation of the calls exposed by each pallet.
		//
		// The parsed function names will be `snake_case`, and that will show up in the enum.
		#[allow(non_camel_case_types)]
		#[derive(Debug, Clone, serde::Serialize)]
		pub enum RuntimeCall {
			// Generate a variant for each callable pallet
			#( #pallet_names(#pallet_names::Call<#runtime_struct>) ),*
		}

		// Implement the Dispatch trait for the Runtime
		impl crate::support::Dispatch for #runtime_struct {
			// The Caller type is the AccountId from the system Config
			type Caller = <Runtime as system::Config>::AccountId;
			// The Call type is the RuntimeCall enum we just defined
			type Call = RuntimeCall;
			// Dispatch a call on behalf of a caller. Increments the caller's nonce.
			//
			// Dispatch allows us to identify which underlying pallet call we want to execute.
			// Note that we extract the `caller` from the extrinsic, and use that information
			// to determine who we are executing the call on behalf of.
			fn dispatch(
				&mut self,
				caller: Self::Caller,
				runtime_call: Self::Call,
			) -> crate::support::DispatchResult {
				// This match statement will allow us to correctly route `RuntimeCall`s
				// to the appropriate pallet level call.
				match runtime_call {
					#(
						// Route each call variant to its pallet
						RuntimeCall::#pallet_names(call) => {
							// Delegate to the pallet's dispatch implementation
							self.#pallet_names.dispatch(caller, call)?;
						}
					),*
				}
				// Return success after the call completes
				Ok(())
			}
		}
	};

	// We combine and return all the generated code.
	quote! {
		// Include the dispatch implementation
		#dispatch_impl
		// Include the runtime implementation
		#runtime_impl
	}
	.into()
}
