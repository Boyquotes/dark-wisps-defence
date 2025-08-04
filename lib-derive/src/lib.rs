use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields, FieldsUnnamed};

/// Derives the Property trait for structs containing a single f32 field.
/// 
/// This macro automatically implements `get()` and `set()` methods that
/// access the first (and typically only) field of a newtype struct.
///
/// # Example
/// ```rust
/// #[derive(Property)]
/// struct Temperature(f32);
/// 
/// // Generates:
/// // impl Property for Temperature {
/// //     fn get(&self) -> f32 { self.0 }
/// //     fn set(&mut self, value: f32) { self.0 = value }
/// // }
/// ```
#[proc_macro_derive(Property)]
pub fn derive_property(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    
    // Generate the implementation based on the struct's fields
    let property_impl = match generate_property_impl(&input.data) {
        Ok(impl_block) => impl_block,
        Err(err) => return err.to_compile_error().into(),
    };
    
    let expanded = quote! {
        impl #impl_generics Property for #name #ty_generics #where_clause {
            #property_impl
        }
    };
    
    TokenStream::from(expanded)
}

fn generate_property_impl(data: &Data) -> syn::Result<TokenStream2> {
    match data {
        Data::Struct(data_struct) => {
            match &data_struct.fields {
                Fields::Unnamed(FieldsUnnamed { unnamed, .. }) if unnamed.len() == 1 => {
                    // Single unnamed field (newtype pattern)
                    Ok(quote! {
                        fn get(&self) -> f32 {
                            self.0
                        }
                        
                        fn set(&mut self, value: f32) {
                            self.0 = value;
                        }
                    })
                }
                Fields::Named(fields) if fields.named.len() == 1 => {
                    // Single named field
                    let field = fields.named.first().unwrap();
                    let field_name = field.ident.as_ref().unwrap();
                    Ok(quote! {
                        fn get(&self) -> f32 {
                            self.#field_name
                        }
                        
                        fn set(&mut self, value: f32) {
                            self.#field_name = value;
                        }
                    })
                }
                _ => Err(syn::Error::new(
                    proc_macro2::Span::call_site(),
                    "Property derive only supports structs with exactly one field"
                ))
            }
        }
        _ => Err(syn::Error::new(
            proc_macro2::Span::call_site(),
            "Property derive only supports structs"
        ))
    }
}
