// Import the CallDef struct from the parse module
use super::parse::CallDef;
// Import the quote macro for code generation
use quote::quote;

// See the `fn call` docs at the `lib.rs` of this crate for a high level definition.
// This function generates the dispatch code for callable functions
pub fn expand_call(def: CallDef) -> proc_macro2::TokenStream {
	// Destructure the CallDef to get its components
	let CallDef { pallet_struct, methods } = def;

	// This is a vector of all the callable function names.
	let fn_name = methods.iter().map(|method| &method.name).collect::<Vec<_>>();

	// This is a nested vector of all the arguments for each of the functions in `fn_name`. It does
	// not include the `self` or `caller: T::AccountId` parameter, which we always assume are the
	// first two parameters to these calls.
	let args_name = methods
		.iter()
		.map(|method| method.args.iter().map(|(name, _)| name.clone()).collect::<Vec<_>>())
		.collect::<Vec<_>>();

	// This is a nested vector of all the types for all the arguments for each of the functions in
	// `fn_name`. It has the same assumptions as `args_name`.
	let args_type = methods
		.iter()
		.map(|method| method.args.iter().map(|(_, type_)| type_.clone()).collect::<Vec<_>>())
		.collect::<Vec<_>>();

	// This quote block creates an `enum Call` which contains all the calls exposed by our pallet,
	// and the `Dispatch` trait logic to route a `caller` to access those functions.
	let dispatch_impl = quote! {
		// The callable functions exposed by this pallet.
		//
		// The parsed function names will be `snake_case`, and that will show up in the enum.
		#[allow(non_camel_case_types)]
		#[derive(Debug, Clone, serde::Serialize)]
		pub enum Call<T: Config> {
			// Generate a variant for each callable function
			#(
				// Each variant has the same name as the function and contains its arguments
				#fn_name { #( #args_name: #args_type),* },
			)*
		}

		// Dispatch logic at the pallet level, mapping each of the items in the `Call` enum to the
		// appropriate function call with all arguments, including the `caller`.
		impl<T: Config> crate::support::Dispatch for #pallet_struct<T> {
			// The Caller type is the AccountId from the Config
			type Caller = T::AccountId;
			// The Call type is the Call enum we just defined
			type Call = Call<T>;

			// The dispatch function routes calls to the appropriate function
			fn dispatch(&mut self, caller: Self::Caller, call: Self::Call) -> crate::support::DispatchResult {
				// Match on the call to determine which function to execute
				match call {
					#(
						// Pattern match on each call variant
						Call::#fn_name { #( #args_name ),* } => {
							// Call the actual function with caller and arguments
							self.#fn_name(
								// Note that we assume the first argument of every call is the `caller`.
								caller,
								// Pass the remaining arguments
								#( #args_name ),*
							)?;
						},
					)*
				}
				// Return success after the call completes
				Ok(())
			}
		}
	};

	// Return the generated code.
	dispatch_impl.into()
}
