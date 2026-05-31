// Declare the expand module for code generation
pub mod expand;
// Declare the parse module for parsing the input
pub mod parse;

// See the `fn call` docs at the `lib.rs` of this crate for a high level definition.
// This function implements the call macro that processes callable functions in a pallet
pub fn call(
	// The attributes passed to the macro (currently unused)
	_attr: proc_macro::TokenStream,
	// The item (impl block) that the macro is applied to
	item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
	// The final expanded code will be placed here.
	// Since our macro only adds new code, our final product will contain all of our old code too,
	// hence we clone `item`.
	let mut finished = item.clone();
	// Parse the input item into a syntax node
	let item_mod = syn::parse_macro_input!(item as syn::Item);

	// First we parse the call functions implemented for the pallet...
	let generated: proc_macro::TokenStream = match parse::CallDef::try_from(item_mod.clone()) {
		// ..then we generate our new code.
		Ok(def) => expand::expand_call(def).into(),
		// If parsing fails, convert the error to a compile error
		Err(e) => e.to_compile_error().into(),
	};

	// Add our generated code to the end, and return the final result.
	finished.extend(generated);
	// Return the complete expanded code
	return finished;
}
