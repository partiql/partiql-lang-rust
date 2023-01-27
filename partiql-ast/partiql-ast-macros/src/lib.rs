use darling::{FromDeriveInput, FromField, FromVariant};
use inflector::Inflector;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{Data, Fields};
use syn::{DeriveInput, Ident};

#[proc_macro_derive(Visit, attributes(visit))]
pub fn visit_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = syn::parse_macro_input!(input as syn::DeriveInput);
    let gen = impl_visit(&ast);
    gen.into()
}

#[derive(FromDeriveInput)]
#[darling(attributes(visit))]
struct VisitItemOptions {
    #[darling(rename = "skip_recurse")]
    should_skip_recurse: Option<bool>,
}

fn should_skip_recurse(input: &syn::DeriveInput) -> bool {
    VisitItemOptions::from_derive_input(input)
        .expect("parse meta")
        .should_skip_recurse
        .unwrap_or(false)
}

#[derive(FromField, FromVariant)]
#[darling(attributes(visit))]
struct VisitFieldOptions {
    #[darling(rename = "skip")]
    should_skip: Option<bool>,
}

fn should_skip_field(field: &syn::Field) -> bool {
    VisitFieldOptions::from_field(field)
        .expect("parse meta")
        .should_skip
        .unwrap_or(false)
}

fn should_skip_variant(variant: &syn::Variant) -> bool {
    VisitFieldOptions::from_variant(variant)
        .expect("parse meta")
        .should_skip
        .unwrap_or(false)
}

fn impl_visit(ast: &syn::DeriveInput) -> proc_macro2::TokenStream {
    let visit_fn_name = &ast.ident.to_string().to_snake_case();
    let enter_fn_name = Ident::new(
        &format!("enter_{visit_fn_name}"),
        proc_macro2::Span::call_site(),
    );
    let exit_fn_name = Ident::new(
        &format!("exit_{visit_fn_name}"),
        proc_macro2::Span::call_site(),
    );

    let visit_children = (!should_skip_recurse(ast)).then(|| impl_visit_children(&ast));

    let ast_name = &ast.ident;
    quote! {
        impl crate::visit::Visit for #ast_name {
            fn visit<'v, V>(&'v self, v: &mut V)
            where
                V: crate::visit::Visitor<'v>,
            {
                v.#enter_fn_name(self);
                #visit_children;
                v.#exit_fn_name(self);
            }
        }
    }
}

fn impl_visit_children(ast: &&DeriveInput) -> TokenStream {
    match &ast.data {
        Data::Enum(e) => {
            let enum_name = std::iter::repeat(&ast.ident);
            let variants = {
                e.variants
                    .iter()
                    .filter_map(|v| (!should_skip_variant(v)).then_some(&v.ident))
            };

            let variants = variants.collect::<Vec<_>>();
            let non_exhaustive = variants.len() < e.variants.len();
            let else_clause = non_exhaustive.then(|| {
                quote! {
                    _ => {}
                }
            });

            quote! {
                match &self {
                    #(#enum_name::#variants(child) => child.visit(v),)*
                    #else_clause
                }
            }
        }
        Data::Struct(s) => {
            let fields: Vec<_> = match &s.fields {
                Fields::Named(named) => named
                    .named
                    .iter()
                    .filter_map(|f| {
                        if should_skip_field(f) {
                            None
                        } else {
                            f.ident.clone()
                        }
                    })
                    .collect(),
                Fields::Unnamed(unnamed) => unnamed
                    .unnamed
                    .iter()
                    .enumerate()
                    .filter_map(|(i, f)| {
                        if should_skip_field(f) {
                            None
                        } else {
                            Some(format_ident!("{}", i))
                        }
                    })
                    .collect(),
                Fields::Unit => vec![],
            };
            quote! {
                #(self.#fields.visit(v);)*
            }
        }
        Data::Union(_) => panic!("Union not supported"),
    }
}
