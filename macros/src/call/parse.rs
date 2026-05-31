// Import the ToTokens trait from quote for converting syntax nodes to tokens
use quote::ToTokens;
// Import the Spanned trait from syn for error handling
use syn::spanned::Spanned;

// Custom keywords we match to when parsing the calls in a pallet.
// This module defines custom syntax keywords for our macro DSL
mod keyword {
	// Define the "T" keyword for generic type parameters
	syn::custom_keyword!(T);
	// Define the "AccountId" keyword for the account identifier type
	syn::custom_keyword!(AccountId);
}

// This object will collect all the information we need to keep while parsing the callable
// functions.
#[derive(Debug)]
pub struct CallDef {
	/// This is the name of the pallet struct where the callable functions are implemented. We
	/// mostly assume it is `Pallet`.
	pub pallet_struct: syn::Ident,
	/// This is a list of the callable functions exposed by this pallet. See `CallVariantDef`.
	pub methods: Vec<CallVariantDef>,
}

// This is the metadata we keep about each callable function in our pallet.
#[derive(Debug)]
pub struct CallVariantDef {
	/// The function name.
	pub name: syn::Ident,
	/// Information on args of the function: `(name, type)`.
	pub args: Vec<(syn::Ident, Box<syn::Type>)>,
}

// Implementation for parsing an Item into a CallDef
impl CallDef {
	// Try to parse a syntax Item into a CallDef
	pub fn try_from(item: syn::Item) -> syn::Result<Self> {
		// First we check that we are parsing an `impl`.
		let item_impl = if let syn::Item::Impl(item) = item {
			// Item is an impl block, use it
			item
		} else {
			// Item is not an impl block, return an error
			return Err(syn::Error::new(item.span(), "Invalid pallet::call, expected item impl"))
		};

		// Extract the name of the struct. We mostly assume it is `Pallet`, but we can handle it
		// when it isn't.
		let pallet_struct = match &*item_impl.self_ty {
			// Self type is a path (e.g., Pallet<T>)
			syn::Type::Path(tp) => {
				// Get the first segment of the path (the struct name)
				tp.path.segments.first().unwrap().ident.clone()
			},
			// Self type is not a path, panic (shouldn't happen)
			_ => panic!("not supported tokens"),
		};

		// Here is where we will store all the callable functions.
		let mut methods = vec![];
		// Iterate over all items in the impl block
		for item in item_impl.items {
			// We only care about functions (methods)
			if let syn::ImplItem::Fn(method) = item {
				// Here is where we will store all the args for each callable functions.
				let mut args = vec![];

				// First argument should be some variant of `self`.
				match method.sig.inputs.first() {
					// First argument is a receiver (self, &self, or &mut self) - good!
					Some(syn::FnArg::Receiver(_)) => {},
					// First argument is not a receiver - error!
					_ => {
						// Return an error indicating the first argument must be self
						let msg = "Invalid call, first argument must be a variant of self";
						return Err(syn::Error::new(method.sig.span(), msg))
					},
				}

				// The second argument should be the `caller: T::AccountId` argument.
				match method.sig.inputs.iter().skip(1).next() {
					// Second argument exists and is typed
					Some(syn::FnArg::Typed(arg)) => {
						// Here we specifically check that this argument is as we expect for
						// `caller: T::AccountId`.
						check_caller_arg(arg)?;
					},
					// Second argument doesn't exist or isn't typed - error!
					_ => {
						// Return an error indicating the second argument must be caller
						let msg = "Invalid call, second argument should be `caller: T::AccountId`";
						return Err(syn::Error::new(method.sig.span(), msg))
					},
				}

				// Get the function name
				let fn_name = method.sig.ident.clone();

				// Parsing the rest of the args. Skipping 2 for `self` and `caller`.
				for arg in method.sig.inputs.iter().skip(2) {
					// All arguments should be typed.
					let arg = if let syn::FnArg::Typed(arg) = arg {
						// This is a typed argument, use it
						arg
					} else {
						// This is not a typed argument (shouldn't happen)
						unreachable!("All args should be typed.");
					};

					// Extract the name of the argument.
					let arg_ident = if let syn::Pat::Ident(pat) = &*arg.pat {
						// Pattern is an identifier, get the name
						pat.ident.clone()
					} else {
						// Pattern is not an identifier - error!
						let msg = "Invalid pallet::call, argument must be ident";
						return Err(syn::Error::new(arg.pat.span(), msg))
					};

					// Store the argument name and the argument type for generating code.
					args.push((arg_ident, arg.ty.clone()));
				}

				// Store all the function name and the arg data for the function.
				methods.push(CallVariantDef { name: fn_name, args });
			}
		}

		// Return all callable functions for this pallet.
		Ok(Self { pallet_struct, methods })
	}
}

// Check caller arg is exactly: `caller: T::AccountId`.
//
// This is kept strict to keep the code simple.
pub fn check_caller_arg(arg: &syn::PatType) -> syn::Result<()> {
	// Helper struct for parsing the T::AccountId type
	pub struct CheckDispatchableFirstArg;
	// Implement Parse trait for our helper struct
	impl syn::parse::Parse for CheckDispatchableFirstArg {
		// Parse the input stream
		fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
			// Parse the "T" keyword
			input.parse::<keyword::T>()?;
			// Parse the "::" token
			input.parse::<syn::Token![::]>()?;
			// Parse the "AccountId" keyword
			input.parse::<keyword::AccountId>()?;
			// Return success
			Ok(Self)
		}
	}

	// This checks the arg name is `caller` or `_caller`.
	if let syn::Pat::Ident(ident) = &*arg.pat {
		// We also support the name as `_caller` for when the variable is unused.
		if &ident.ident != "caller" && &ident.ident != "_caller" {
			// Argument name is not caller or _caller - error!
			let msg = "Invalid name for second parameter: expected `caller: T::AccountId`";
			return Err(syn::Error::new(ident.span(), msg))
		}
	}

	// This checks the type is `T::AccountId` with `CheckDispatchableFirstArg`
	let ty = &arg.ty;
	// Try to parse the type as T::AccountId
	syn::parse2::<CheckDispatchableFirstArg>(ty.to_token_stream()).map_err(|e| {
		// Parsing failed, create an error message
		let msg = "Invalid type for second parameter: expected `caller: T::AccountId`";
		let mut err = syn::Error::new(ty.span(), msg);
		// Combine the original parse error with our message
		err.combine(e);
		// Return the combined error
		err
	})?;

	// Return success
	Ok(())
}
