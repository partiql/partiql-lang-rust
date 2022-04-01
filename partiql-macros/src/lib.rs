extern crate proc_macro;
use proc_macro::{TokenStream};
use quote::{quote};
use syn::DeriveInput;

// using proc_macro_attribute to declare an attribute like procedural macro
#[proc_macro_attribute]
// _metadata is argument provided to macro call and _input is code to which attribute like macro attaches
pub fn my_custom_attribute(_metadata: TokenStream, _input: TokenStream) -> TokenStream {
    // returing a simple TokenStream for Struct
    TokenStream::from(quote!{struct H{}})
}

#[proc_macro_derive(AstNodeBuilder)]
pub fn derive_ast(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as DeriveInput);
    let name = input.ident;
    let expanded = quote! {
        impl AstNodeBuilder for #name {}
    };

    proc_macro::TokenStream::from(expanded)
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
