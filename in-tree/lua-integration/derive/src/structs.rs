use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, Fields, Result};

use crate::attrs::ContainerAttrs;

pub fn expand_struct(input: DeriveInput) -> TokenStream {
    match expand_struct_impl(input) {
        Ok(tokens) => tokens,
        Err(err) => err.to_compile_error().into(),
    }
}

fn expand_struct_impl(input: DeriveInput) -> Result<TokenStream> {
    let attrs = ContainerAttrs::from_derive_input(&input)?;

    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let fields = match &input.data {
        syn::Data::Struct(data) => &data.fields,
        _ => unreachable!(),
    };

    let field_methods = generate_field_methods(fields)?;
    let meta_methods = generate_meta_methods(&attrs, name)?;

    let mut where_clause_with_bounds = where_clause.cloned();

    if attrs.eq {
        let predicate: syn::WherePredicate = syn::parse_quote!(#name #ty_generics: ::std::cmp::PartialEq);
        where_clause_with_bounds
            .get_or_insert_with(|| syn::parse_quote!(where))
            .predicates
            .push(predicate);
    }

    let expanded = quote! {
        impl #impl_generics ::mlua::UserData for #name #ty_generics #where_clause_with_bounds {
            fn add_fields<F: ::mlua::UserDataFields<Self>>(fields: &mut F) {
                #field_methods
            }

            fn add_methods<M: ::mlua::UserDataMethods<Self>>(methods: &mut M) {
                #meta_methods
            }
        }
    };

    Ok(expanded.into())
}

fn generate_field_methods(fields: &Fields) -> Result<proc_macro2::TokenStream> {
    let mut tokens = proc_macro2::TokenStream::new();

    match fields {
        Fields::Named(fields) => {
            for field in &fields.named {
                let vis = &field.vis;
                if !matches!(vis, syn::Visibility::Public(_)) {
                    continue;
                }

                let field_name = field.ident.as_ref().unwrap();
                let field_name_str = field_name.to_string();
                let field_type = &field.ty;

                tokens.extend(quote! {
                    fields.add_field_method_get(#field_name_str, |_lua, this| {
                        Ok(this.#field_name.clone())
                    });

                    fields.add_field_method_set(#field_name_str, |_lua, this, value: #field_type| {
                        this.#field_name = value;
                        Ok(())
                    });
                });
            }
        }
        Fields::Unnamed(_) => {
            return Err(syn::Error::new_spanned(
                fields,
                "Tuple structs are not supported by LuaIntegration"
            ));
        }
        Fields::Unit => {}
    }

    Ok(tokens)
}

fn generate_meta_methods(attrs: &ContainerAttrs, name: &syn::Ident) -> Result<proc_macro2::TokenStream> {
    let mut tokens = proc_macro2::TokenStream::new();

    if attrs.serde {
        tokens.extend(quote! {
            methods.add_meta_method(::mlua::MetaMethod::ToString, |_lua, this, ()| {
                serde_json::to_string(this)
                    .map_err(|e| ::mlua::Error::RuntimeError(format!("Failed to serialize: {}", e)))
            });

            methods.add_function("from_json", |_lua, json_str: String| {
                serde_json::from_str::<#name>(&json_str)
                    .map_err(|e| ::mlua::Error::RuntimeError(format!("Failed to deserialize: {}", e)))
            });
        });
    }

    if attrs.eq {
        tokens.extend(quote! {
            methods.add_meta_method(::mlua::MetaMethod::Eq, |_lua, this, other: ::mlua::AnyUserData| {
                if let Ok(other_val) = other.borrow::<#name>() {
                    Ok(this == &*other_val)
                } else {
                    Ok(false)
                }
            });
        });
    }

    Ok(tokens)
}
