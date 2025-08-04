use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields, FieldsUnnamed, Attribute, Meta};

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
                        
                        fn new(value: f32) -> Self {
                            Self(value)
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

                        fn new(value: f32) -> Self {
                            Self { #field_name: value }
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

/// Derives the Modifier trait for structs with a #[require(ModifierType = ...)] attribute.
/// 
/// This macro automatically implements the Modifier trait by extracting the MODIFIER_TYPE
/// from the required ModifierType attribute, eliminating duplication.
///
/// # Example
/// ```rust
/// #[derive(Component, Clone, Default, Property, Modifier)]
/// #[component(immutable)]
/// #[require(ModifierType = ModifierType::AttackSpeed)]
/// pub struct ModifierAttackSpeed(pub f32);
/// 
/// // Generates:
/// // impl Modifier for ModifierAttackSpeed {
/// //     const MODIFIER_TYPE: ModifierType = ModifierType::AttackSpeed;
/// // }
/// ```
#[proc_macro_derive(Modifier)]
pub fn derive_modifier(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    
    // Extract MODIFIER_TYPE from #[require(ModifierType = ...)] attribute
    let modifier_type = match extract_modifier_type(&input.attrs) {
        Ok(modifier_type) => modifier_type,
        Err(err) => return err.to_compile_error().into(),
    };
    
    let expanded = quote! {
        impl #impl_generics Modifier for #name #ty_generics #where_clause {
            const MODIFIER_TYPE: ModifierType = #modifier_type;
        }
    };
    
    TokenStream::from(expanded)
}

fn extract_modifier_type(attrs: &[Attribute]) -> syn::Result<proc_macro2::TokenStream> {
    for attr in attrs {
        if attr.path().is_ident("require") {
            match &attr.meta {
                Meta::List(meta_list) => {
                    // Parse the tokens inside #[require(...)]
                    let tokens = &meta_list.tokens;
                    let tokens_str = tokens.to_string();
                    
                    // Look for ModifierType = ... pattern
                    if tokens_str.starts_with("ModifierType =") {
                        // Extract the value after the equals sign
                        let value_part = tokens_str.strip_prefix("ModifierType =")
                            .unwrap()
                            .trim();
                        
                        // Parse the value as a token stream
                        let modifier_type: proc_macro2::TokenStream = value_part.parse()
                            .map_err(|_| syn::Error::new_spanned(attr, "Invalid ModifierType value"))?;
                        
                        return Ok(modifier_type);
                    }
                }
                _ => continue,
            }
        }
    }
    
    Err(syn::Error::new(
        proc_macro2::Span::call_site(),
        "Modifier derive requires a #[require(ModifierType = ...)] attribute"
    ))
}
