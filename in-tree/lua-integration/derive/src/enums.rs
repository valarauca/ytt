use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, Result, Variant, Fields};

use crate::attrs::{VariantAttrs, ContainerAttrs};

pub fn expand_enum(input: DeriveInput) -> TokenStream {
    match expand_enum_impl(input) {
        Ok(tokens) => tokens,
        Err(err) => err.to_compile_error().into(),
    }
}

fn expand_enum_impl(input: DeriveInput) -> Result<TokenStream> {
    let attrs = ContainerAttrs::from_derive_input(&input)?;
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let variants = match &input.data {
        syn::Data::Enum(data) => &data.variants,
        _ => unreachable!(),
    };

    let is_unit_enum = variants.iter().all(|v| matches!(v.fields, Fields::Unit));

    let variant_methods = generate_variant_methods(name, variants)?;
    let eq_method = if is_unit_enum {
        generate_unit_enum_eq(name, variants)?
    } else {
        generate_discriminant_eq(name)?
    };
    let serde_methods = generate_serde_methods(&attrs, name)?;

    let expanded = quote! {
        impl #impl_generics ::mlua::UserData for #name #ty_generics #where_clause {
            fn add_methods<M: ::mlua::UserDataMethods<Self>>(methods: &mut M) {
                #eq_method
                #variant_methods
                #serde_methods
            }
        }
    };

    Ok(expanded.into())
}

fn generate_variant_methods(enum_name: &syn::Ident, variants: &syn::punctuated::Punctuated<Variant, syn::token::Comma>) -> Result<proc_macro2::TokenStream> {
    let mut tokens = proc_macro2::TokenStream::new();

    for variant in variants {
        let variant_name = &variant.ident;
        let variant_attrs = VariantAttrs::from_attributes(&variant.attrs)?;
        let lua_name = variant_attrs.rename.as_ref().unwrap_or(&variant_name.to_string()).clone();

        let is_method_name = format!("is_{}", to_snake_case(&lua_name));

        match &variant.fields {
            Fields::Unit => {
                tokens.extend(quote! {
                    methods.add_method(#is_method_name, |_lua, this, ()| {
                        Ok(matches!(this, #enum_name::#variant_name))
                    });
                });
            }
            Fields::Unnamed(fields) => {
                tokens.extend(quote! {
                    methods.add_method(#is_method_name, |_lua, this, ()| {
                        Ok(matches!(this, #enum_name::#variant_name(..)))
                    });
                });

                if fields.unnamed.len() == 1 {
                    let get_method_name = format!("get_{}", to_snake_case(&lua_name));

                    tokens.extend(quote! {
                        methods.add_method(#get_method_name, |_lua, this, ()| {
                            if let #enum_name::#variant_name(value) = this {
                                Ok(Some(value.clone()))
                            } else {
                                Ok(None)
                            }
                        });
                    });
                } else {
                    for (idx, _field) in fields.unnamed.iter().enumerate() {
                        let get_method_name = format!("get_{}_{}", to_snake_case(&lua_name), idx);

                        let pattern_fields: Vec<_> = (0..fields.unnamed.len())
                            .map(|i| {
                                if i == idx {
                                    quote! { ref value }
                                } else {
                                    quote! { _ }
                                }
                            })
                            .collect();

                        tokens.extend(quote! {
                            methods.add_method(#get_method_name, |_lua, this, ()| {
                                if let #enum_name::#variant_name(#(#pattern_fields),*) = this {
                                    Ok(Some(value.clone()))
                                } else {
                                    Ok(None)
                                }
                            });
                        });
                    }
                }
            }
            Fields::Named(fields) => {
                tokens.extend(quote! {
                    methods.add_method(#is_method_name, |_lua, this, ()| {
                        Ok(matches!(this, #enum_name::#variant_name { .. }))
                    });
                });

                for field in &fields.named {
                    let field_name = field.ident.as_ref().unwrap();
                    let get_method_name = format!("get_{}_{}", to_snake_case(&lua_name), field_name);

                    tokens.extend(quote! {
                        methods.add_method(#get_method_name, |_lua, this, ()| {
                            if let #enum_name::#variant_name { #field_name, .. } = this {
                                Ok(Some(#field_name.clone()))
                            } else {
                                Ok(None)
                            }
                        });
                    });
                }
            }
        }
    }

    Ok(tokens)
}

fn generate_unit_enum_eq(enum_name: &syn::Ident, variants: &syn::punctuated::Punctuated<Variant, syn::token::Comma>) -> Result<proc_macro2::TokenStream> {
    let mut match_arms = proc_macro2::TokenStream::new();

    for variant in variants {
        let variant_name = &variant.ident;
        let variant_attrs = VariantAttrs::from_attributes(&variant.attrs)?;
        let lua_name = variant_attrs.rename.as_ref().unwrap_or(&variant_name.to_string()).clone();

        match_arms.extend(quote! {
            #enum_name::#variant_name => s == #lua_name,
        });
    }

    let tokens = quote! {
        methods.add_meta_method(::mlua::MetaMethod::Eq, |_lua, this, other: ::mlua::Value| {
            if let ::mlua::Value::String(s) = &other {
                let s = s.to_str()?;
                let result = match this {
                    #match_arms
                };
                return Ok(result);
            }

            if let ::mlua::Value::UserData(ud) = &other {
                if let Ok(other_val) = ud.borrow::<#enum_name>() {
                    return Ok(std::mem::discriminant(this) == std::mem::discriminant(&*other_val));
                }
            }

            Ok(false)
        });
    };

    Ok(tokens)
}

fn generate_discriminant_eq(enum_name: &syn::Ident) -> Result<proc_macro2::TokenStream> {
    let tokens = quote! {
        methods.add_meta_method(::mlua::MetaMethod::Eq, |_lua, this, other: ::mlua::Value| {
            if let ::mlua::Value::UserData(ud) = &other {
                if let Ok(other_val) = ud.borrow::<#enum_name>() {
                    return Ok(std::mem::discriminant(this) == std::mem::discriminant(&*other_val));
                }
            }
            Ok(false)
        });
    };

    Ok(tokens)
}

fn generate_serde_methods(attrs: &ContainerAttrs, name: &syn::Ident) -> Result<proc_macro2::TokenStream> {
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

    Ok(tokens)
}

fn to_snake_case(s: &str) -> String {
    let mut result = String::new();

    for ch in s.chars() {
        if ch.is_uppercase() {
            if !result.is_empty() {
                result.push('_');
            }
            result.push(ch.to_lowercase().next().unwrap());
        } else {
            result.push(ch);
        }
    }

    result
}
