//! Procedural macro implementation for **AutoMorphic**.
//!
//! This crate provides `#[derive(AutoMorph)]`, which generates `impl Morph<T>`
//! and optionally `impl MorphResult<T>` for structs and enums.
//!
//! # Usage
//!
//! ```ignore
//! use auto_morphic::{Morph, AutoMorph};
//!
//! #[derive(AutoMorph)]
//! #[map_to(User)]
//! struct UserDto {
//!     #[map(name = "username")]
//!     name: String,
//! }
//! ```
//!
//! # Attributes
//!
//! ## Struct-level: `#[map_to(Tipo)]`
//!
//! One or more `#[map_to(...)]` attributes specify the target type(s).
//! Each target generates a separate `impl Morph<T>` block.
//!
//! ## Field-level: `#[map(...)]`
//!
//! | Option | Description |
//! |---|---|
//! | `name = "field"` | Destination field name |
//! | `map_to_with = fn` | Transform for `morph_to` |
//! | `map_from_with = fn` | Transform for `morph_from` |
//! | `map_to_result_with = fn` | Fallible transform for `morph_to_result` |
//! | `map_from_result_with = fn` | Fallible transform for `morph_from_result` |
//! | `skip` | Omit this field |
//! | `default` | Use `Default::default()` |
//! | `default = expr` | Custom default expression |
//! | `default_from = expr` | Default only for `morph_from` |
//! | `flatten` | Flatten a nested struct |
//!
//! # Enum Support
//!
//! Unit enum variants are mapped by name. Use `#[map(name = "Variant")]` to
//! rename a variant in the target enum.
//!
//! # Serde Integration
//!
//! With the `serde-integration` feature enabled, `#[serde(rename = "...")]`
//! is used as a fallback when `#[map(name = "...")]` is absent.

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse2, Data, DeriveInput, Field, Fields, GenericParam, Ident, Path, LitStr, Meta, Token, Expr, Type, Variant};

/// Check if a syn::Path represents `Default::default`
fn is_default_default(path: &Path) -> bool {
    let mut segments = path.segments.iter();
    if let Some(first) = segments.next()
        && first.ident == "Default"
        && let Some(second) = segments.next()
        && second.ident == "default"
    {
        return segments.next().is_none();
    }
    false
}

/// Check if a type is a known Copy type (primitives).
/// This is a heuristic: we check the type name against known Copy types.
fn is_copy_type(ty: &Type) -> bool {
    match ty {
        Type::Path(type_path) => {
            if let Some(segment) = type_path.path.segments.last() {
                let name = segment.ident.to_string();
                // Common Copy types in Rust
                matches!(
                    name.as_str(),
                    "bool" | "u8" | "u16" | "u32" | "u64" | "u128" | "usize"
                        | "i8" | "i16" | "i32" | "i64" | "i128" | "isize"
                        | "f32" | "f64" | "char"
                )
            } else {
                false
            }
        }
        Type::Reference(_) => {
            // References are always Copy
            true
        }
        _ => false,
    }
}

/// Extract the inner type from Option<T> if this type is Option<T>, otherwise return None.
fn extract_option_inner(ty: &Type) -> Option<&Type> {
    if let Type::Path(type_path) = ty
        && let Some(segment) = type_path.path.segments.last()
        && segment.ident == "Option"
        && let syn::PathArguments::AngleBracketed(args) = &segment.arguments
        && let Some(syn::GenericArgument::Type(inner_ty)) = args.args.first()
    {
        return Some(inner_ty);
    }
    None
}

/// Check if a type is Option<T>
fn is_option_type(ty: &Type) -> bool {
    extract_option_inner(ty).is_some()
}

/// Collect all type parameter names from generics
fn collect_type_param_names(generics: &syn::Generics) -> Vec<String> {
    generics
        .params
        .iter()
        .filter_map(|param| {
            if let GenericParam::Type(ty_param) = param {
                Some(ty_param.ident.to_string())
            } else {
                None
            }
        })
        .collect()
}

/// Field mapping information for struct fields
struct InfoColumn {
    ident_column: Ident,
    column_name: String,
    field_type: Type,
    fn_map_to_with: Option<Path>,
    fn_map_from_with: Option<Path>,
    fn_map_to_result_with: Option<Path>,
    fn_map_from_result_with: Option<Path>,
    skip_to: bool,
    skip_from: bool,
    default_to: Option<Expr>,
    default_from: Option<Expr>,
    flatten_to: bool,
    flatten_from: bool,
}

/// Parse #[map(...)] attributes on a field, returning the field info
fn parse_field_map_attrs(field: &Field) -> syn::Result<InfoColumn> {
    let ident_column = match &field.ident {
        Some(ident) => ident.clone(),
        None => {
            return Err(syn::Error::new_spanned(
                &field.ty,
                "AutoMorph requires named fields; tuple-style fields are not supported",
            ));
        }
    };
    let mut column_name = ident_column.to_string();
    let mut fn_map_to_with: Option<Path> = None;
    let mut fn_map_from_with: Option<Path> = None;
    let mut fn_map_to_result_with: Option<Path> = None;
    let mut fn_map_from_result_with: Option<Path> = None;
    let mut skip_to = false;
    let mut skip_from = false;
    let mut default_to: Option<Expr> = None;
    let mut default_from: Option<Expr> = None;
    let mut flatten_to = false;
    let mut flatten_from = false;

    for attr in &field.attrs {
        // Handle #[map(...)] attributes
        if attr.path().is_ident("map")
            && let Ok(meta_list) = attr.parse_args_with(syn::punctuated::Punctuated::<Meta, Token![,]>::parse_terminated)
        {
            for meta in meta_list {
                match &meta {
                    Meta::NameValue(nv) if nv.path.is_ident("name") => {
                        if let syn::Expr::Lit(syn::ExprLit { lit: syn::Lit::Str(lit_str), .. }) = &nv.value {
                            column_name = lit_str.value();
                        }
                    }
                    Meta::NameValue(nv) if nv.path.is_ident("map_to_with") => {
                        if let syn::Expr::Path(expr_path) = &nv.value {
                            fn_map_to_with = Some(expr_path.path.clone());
                        }
                    }
                    Meta::NameValue(nv) if nv.path.is_ident("map_from_with") => {
                        if let syn::Expr::Path(expr_path) = &nv.value {
                            fn_map_from_with = Some(expr_path.path.clone());
                        }
                    }
                    Meta::NameValue(nv) if nv.path.is_ident("map_to_result_with") => {
                        if let syn::Expr::Path(expr_path) = &nv.value {
                            fn_map_to_result_with = Some(expr_path.path.clone());
                        }
                    }
                    Meta::NameValue(nv) if nv.path.is_ident("map_from_result_with") => {
                        if let syn::Expr::Path(expr_path) = &nv.value {
                            fn_map_from_result_with = Some(expr_path.path.clone());
                        }
                    }
                    Meta::NameValue(nv) if nv.path.is_ident("default") => {
                        default_to = Some(nv.value.clone());
                    }
                    Meta::NameValue(nv) if nv.path.is_ident("default_from") => {
                        default_from = Some(nv.value.clone());
                    }
                    Meta::Path(path) if path.is_ident("skip") => {
                        skip_to = true;
                        skip_from = true;
                    }
                    Meta::Path(path) if path.is_ident("default") => {
                        default_to = Some(Expr::Path(syn::ExprPath {
                            attrs: vec![],
                            qself: None,
                            path: syn::parse_quote!(Default::default),
                        }));
                        default_from = Some(Expr::Path(syn::ExprPath {
                            attrs: vec![],
                            qself: None,
                            path: syn::parse_quote!(Default::default),
                        }));
                    }
                    Meta::Path(path) if path.is_ident("flatten") => {
                        flatten_to = true;
                        flatten_from = true;
                    }
                    _ => {
                        return Err(syn::Error::new_spanned(
                            meta,
                            "Unknown attribute in #[map(...)]. Valid options are: name, map_to_with, map_from_with, map_to_result_with, map_from_result_with, skip, default, default_from, flatten",
                        ));
                    }
                }
            }
        }

        // Serde integration: read #[serde(rename = "...")] when feature is enabled
        #[cfg(feature = "serde-integration")]
        if attr.path().is_ident("serde")
            && let Ok(meta_list) = attr.parse_args_with(syn::punctuated::Punctuated::<Meta, Token![,]>::parse_terminated)
        {
            // Only use serde rename if #[map(name = "...")] is not already set
            if column_name == ident_column.to_string() {
                for meta in meta_list {
                    if let Meta::NameValue(nv) = meta {
                        if nv.path.is_ident("rename") {
                            if let syn::Expr::Lit(syn::ExprLit { lit: syn::Lit::Str(lit_str), .. }) = &nv.value {
                                column_name = lit_str.value();
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(InfoColumn {
        ident_column,
        column_name,
        field_type: field.ty.clone(),
        fn_map_to_with,
        fn_map_from_with,
        fn_map_to_result_with,
        fn_map_from_result_with,
        skip_to,
        skip_from,
        default_to,
        default_from,
        flatten_to,
        flatten_from,
    })
}

/// Generate the morph_to expression for a single field
fn generate_morph_to_field(col: &InfoColumn) -> Option<proc_macro2::TokenStream> {
    if col.skip_to {
        return None;
    }

    // Skip flatten fields here — they are handled separately
    if col.flatten_to {
        return None;
    }

    let self_field = &col.ident_column;
    let target_field: Ident = syn::Ident::new(&col.column_name, proc_macro2::Span::call_site());

    if let Some(fn_path) = &col.fn_map_to_with {
        let value_expr = if is_copy_type(&col.field_type) {
            quote! { self.#self_field }
        } else {
            quote! { self.#self_field.clone() }
        };
        Some(quote! {
            #target_field: #fn_path(#value_expr),
        })
    } else if let Some(default_expr) = &col.default_to {
        if let Expr::Path(expr_path) = default_expr {
            if is_default_default(&expr_path.path) {
                Some(quote! {
                    #target_field: Default::default(),
                })
            } else {
                Some(quote! {
                    #target_field: #default_expr,
                })
            }
        } else {
            Some(quote! {
                #target_field: #default_expr,
            })
        }
    } else {
        let source_is_option = is_option_type(&col.field_type);
        if source_is_option {
            let inner_ty = extract_option_inner(&col.field_type).unwrap();
            if is_option_type(inner_ty) {
                Some(quote! {
                    #target_field: self.#self_field.clone(),
                })
            } else {
                Some(quote! {
                    #target_field: self.#self_field.clone().unwrap_or_default(),
                })
            }
        } else {
            let needs_clone = !is_copy_type(&col.field_type);
            if needs_clone {
                Some(quote! {
                    #target_field: self.#self_field.clone(),
                })
            } else {
                Some(quote! {
                    #target_field: self.#self_field,
                })
            }
        }
    }
}

/// Generate the morph_from expression for a single field
fn generate_morph_from_field(col: &InfoColumn) -> Option<proc_macro2::TokenStream> {
    if col.skip_from {
        return None;
    }

    // Skip flatten fields here — they are handled separately
    if col.flatten_from {
        return None;
    }

    let self_field = &col.ident_column;
    let source_field: Ident = syn::Ident::new(&col.column_name, proc_macro2::Span::call_site());

    if let Some(fn_path) = &col.fn_map_from_with {
        Some(quote! {
            #self_field: #fn_path(t.#source_field),
        })
    } else if let Some(default_expr) = &col.default_from {
        if let Expr::Path(expr_path) = default_expr {
            if is_default_default(&expr_path.path) {
                Some(quote! {
                    #self_field: Default::default(),
                })
            } else {
                Some(quote! {
                    #self_field: #default_expr,
                })
            }
        } else {
            Some(quote! {
                #self_field: #default_expr,
            })
        }
    } else {
        let self_is_option = is_option_type(&col.field_type);
        if self_is_option {
            Some(quote! {
                #self_field: Some(t.#source_field),
            })
        } else {
            Some(quote! {
                #self_field: t.#source_field,
            })
        }
    }
}

/// Generate the morph_to_result expression for a single field
fn generate_morph_to_result_field(col: &InfoColumn) -> Option<proc_macro2::TokenStream> {
    if col.skip_to {
        return None;
    }

    // Skip flatten fields here — they are handled separately
    if col.flatten_to {
        return None;
    }

    let self_field = &col.ident_column;
    let target_field: Ident = syn::Ident::new(&col.column_name, proc_macro2::Span::call_site());
    let field_name = &col.column_name;

    if let Some(fn_path) = &col.fn_map_to_result_with {
        let value_expr = if is_copy_type(&col.field_type) {
            quote! { self.#self_field }
        } else {
            quote! { self.#self_field.clone() }
        };
        Some(quote! {
            #target_field: #fn_path(#value_expr).map_err(|e| auto_morphic_core::morph::MorphError::field(#field_name, e.to_string()))?,
        })
    } else if let Some(fn_path) = &col.fn_map_to_with {
        // Fallback to map_to_with if map_to_result_with is not set
        let value_expr = if is_copy_type(&col.field_type) {
            quote! { self.#self_field }
        } else {
            quote! { self.#self_field.clone() }
        };
        Some(quote! {
            #target_field: #fn_path(#value_expr),
        })
    } else if let Some(default_expr) = &col.default_to {
        if let Expr::Path(expr_path) = default_expr {
            if is_default_default(&expr_path.path) {
                Some(quote! {
                    #target_field: Default::default(),
                })
            } else {
                Some(quote! {
                    #target_field: #default_expr,
                })
            }
        } else {
            Some(quote! {
                #target_field: #default_expr,
            })
        }
    } else {
        // For fields without any transform, use the same logic as morph_to
        let source_is_option = is_option_type(&col.field_type);
        if source_is_option {
            let inner_ty = extract_option_inner(&col.field_type).unwrap();
            if is_option_type(inner_ty) {
                Some(quote! {
                    #target_field: self.#self_field.clone(),
                })
            } else {
                Some(quote! {
                    #target_field: self.#self_field.clone().unwrap_or_default(),
                })
            }
        } else {
            let needs_clone = !is_copy_type(&col.field_type);
            if needs_clone {
                Some(quote! {
                    #target_field: self.#self_field.clone(),
                })
            } else {
                Some(quote! {
                    #target_field: self.#self_field,
                })
            }
        }
    }
}

/// Generate the morph_from_result expression for a single field
fn generate_morph_from_result_field(col: &InfoColumn) -> Option<proc_macro2::TokenStream> {
    if col.skip_from {
        return None;
    }

    // Skip flatten fields here — they are handled separately
    if col.flatten_from {
        return None;
    }

    let self_field = &col.ident_column;
    let source_field: Ident = syn::Ident::new(&col.column_name, proc_macro2::Span::call_site());
    let field_name = &col.column_name;

    if let Some(fn_path) = &col.fn_map_from_result_with {
        Some(quote! {
            #self_field: #fn_path(t.#source_field).map_err(|e| auto_morphic_core::morph::MorphError::field(#field_name, e.to_string()))?,
        })
    } else if let Some(fn_path) = &col.fn_map_from_with {
        // Fallback to map_from_with if map_from_result_with is not set
        Some(quote! {
            #self_field: #fn_path(t.#source_field),
        })
    } else if let Some(default_expr) = &col.default_from {
        if let Expr::Path(expr_path) = default_expr {
            if is_default_default(&expr_path.path) {
                Some(quote! {
                    #self_field: Default::default(),
                })
            } else {
                Some(quote! {
                    #self_field: #default_expr,
                })
            }
        } else {
            Some(quote! {
                #self_field: #default_expr,
            })
        }
    } else {
        // For fields without any transform, use the same logic as morph_from
        let self_is_option = is_option_type(&col.field_type);
        if self_is_option {
            Some(quote! {
                #self_field: Some(t.#source_field),
            })
        } else {
            Some(quote! {
                #self_field: t.#source_field,
            })
        }
    }
}

/// Derive macro for automatic `Morph<T>` and `MorphResult<T>` implementation.
///
/// Place `#[map_to(TargetType)]` on the struct or enum to specify the target.
/// Use `#[map(...)]` on fields to configure renaming, transforms, skip, default,
/// and flatten behavior.
///
/// # Example
///
/// ```ignore
/// #[derive(AutoMorph)]
/// #[map_to(User)]
/// struct UserDto {
///     id: i32,
///     #[map(name = "username")]
///     name: String,
///     #[map(skip)]
///     password: String,
/// }
/// ```
#[proc_macro_derive(AutoMorph, attributes(map, map_to))]
pub fn auto_map_derive(input: TokenStream) -> TokenStream {
    match expanded_auto_map_derive(input.into()) {
        Ok(output) => output.into(),
        Err(error) => error.to_compile_error().into(),
    }
}

/// Parse the derive input and generate `impl Morph<T>` (and optionally `impl MorphResult<T>`)
/// for structs and enums.
fn expanded_auto_map_derive(input: proc_macro2::TokenStream) -> syn::Result<proc_macro2::TokenStream> {
    let input: DeriveInput = parse2(input)?;

    let type_name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = &input.generics.split_for_impl();

    // Collect #[map_to(...)] targets
    let mut map_to_structs: Vec<Path> = Vec::new();
    for attr in &input.attrs {
        if attr.path().is_ident("map_to")
            && let Ok(path) = attr.parse_args::<Path>()
        {
            map_to_structs.push(path);
        }
    }

    // Dispatch based on data type
    match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => {
                if map_to_structs.is_empty() {
                    return Err(syn::Error::new_spanned(
                        type_name,
                        "AutoMorph requires at least one #[map_to(TargetStruct)] attribute on the struct\n\
                         Example:\n  #[derive(AutoMorph)]\n  #[map_to(TargetType)]\n  struct MyStruct { ... }",
                    ));
                }
                generate_struct_impls(&input, type_name, fields.named.iter().collect(), &map_to_structs, impl_generics, ty_generics, *where_clause)
            }
            _ => {
                Err(syn::Error::new_spanned(
                    type_name,
                    "AutoMorph can only be derived for structs with named fields (not tuple or unit structs)",
                ))
            }
        },
        Data::Enum(data) => {
            generate_enum_impls(&input, type_name, &data.variants.iter().collect::<Vec<_>>(), &map_to_structs, impl_generics, ty_generics, *where_clause)
        }
        Data::Union(_) => {
            Err(syn::Error::new_spanned(
                type_name,
                "AutoMorph can only be derived for structs and enums, not unions",
            ))
        }
    }
}

/// Generate Morph impl blocks for a struct
fn generate_struct_impls(
    input: &DeriveInput,
    type_name: &Ident,
    fields: Vec<&Field>,
    map_to_structs: &[Path],
    impl_generics: &syn::ImplGenerics,
    ty_generics: &syn::TypeGenerics,
    where_clause: Option<&syn::WhereClause>,
) -> syn::Result<proc_macro2::TokenStream> {
    let columns: Vec<InfoColumn> = fields
        .iter()
        .map(|f| parse_field_map_attrs(f))
        .collect::<syn::Result<Vec<_>>>()?;

    let has_skip_to = columns.iter().any(|col| col.skip_to);
    let has_skip_from = columns.iter().any(|col| col.skip_from);

    let type_param_names = collect_type_param_names(&input.generics);
    let type_param_ids: Vec<Ident> = type_param_names
        .iter()
        .map(|name| syn::Ident::new(name, proc_macro2::Span::call_site()))
        .collect();

    // Build morph_to fields
    let morph_to_fields: Vec<proc_macro2::TokenStream> = columns
        .iter()
        .filter_map(generate_morph_to_field)
        .collect();

    // Build morph_from fields
    let morph_from_fields: Vec<proc_macro2::TokenStream> = columns
        .iter()
        .filter_map(generate_morph_from_field)
        .collect();

    // Build flatten field expansions for morph_to
    // For flatten, the nested struct is morphed to the target's nested struct type
    // using the Morph trait. The target field name matches the source field name.
    let flatten_to_fields: Vec<proc_macro2::TokenStream> = columns
        .iter()
        .filter(|col| col.flatten_to)
        .map(|col| {
            let self_field = &col.ident_column;
            let target_field: Ident = syn::Ident::new(&col.column_name, proc_macro2::Span::call_site());
            quote! {
                #target_field: auto_morphic_core::morph::Morph::morph_to(&self.#self_field),
            }
        })
        .collect();

    // Build flatten field expansions for morph_from
    // For flatten, reconstruct the nested struct from the target's nested struct
    let flatten_from_fields: Vec<proc_macro2::TokenStream> = columns
        .iter()
        .filter(|col| col.flatten_from)
        .map(|col| {
            let self_field = &col.ident_column;
            let source_field: Ident = syn::Ident::new(&col.column_name, proc_macro2::Span::call_site());
            quote! {
                #self_field: auto_morphic_core::morph::Morph::morph_from(t.#source_field),
            }
        })
        .collect();

    // Build morph_to body
    let morph_to_body = {
        let all_to_fields: Vec<proc_macro2::TokenStream> = morph_to_fields
            .into_iter()
            .chain(flatten_to_fields)
            .collect();
        if has_skip_to {
            quote! {
                #(#all_to_fields)*
                ..Default::default()
            }
        } else {
            quote! {
                #(#all_to_fields)*
            }
        }
    };

    // Build morph_from body
    let morph_from_body = {
        let all_from_fields: Vec<proc_macro2::TokenStream> = morph_from_fields
            .into_iter()
            .chain(flatten_from_fields)
            .collect();
        if has_skip_from {
            quote! {
                #(#all_from_fields)*
                ..Default::default()
            }
        } else {
            quote! {
                #(#all_from_fields)*
            }
        }
    };

    // Build enhanced where clause
    let enhanced_where_clause = if !type_param_ids.is_empty() {
        let clone_bounds: Vec<proc_macro2::TokenStream> = type_param_ids.iter().map(|id| {
            quote! { #id: Clone }
        }).collect();

        if let Some(existing_where) = where_clause {
            quote! {
                #existing_where #(#clone_bounds,)*
            }
        } else {
            quote! {
                where #(#clone_bounds),*
            }
        }
    } else {
        quote! {}
    };

    // Check if any field has MorphResult transforms
    let has_result_transform = columns.iter().any(|col| {
        col.fn_map_to_result_with.is_some() || col.fn_map_from_result_with.is_some()
    });

    // Build MorphResult fields if needed
    let morph_result_impls = if has_result_transform {
        let morph_to_result_fields: Vec<proc_macro2::TokenStream> = columns
            .iter()
            .filter_map(generate_morph_to_result_field)
            .collect();
        let morph_from_result_fields: Vec<proc_macro2::TokenStream> = columns
            .iter()
            .filter_map(generate_morph_from_result_field)
            .collect();

        // Build flatten field expansions for MorphResult
        let flatten_to_result_fields: Vec<proc_macro2::TokenStream> = columns
            .iter()
            .filter(|col| col.flatten_to)
            .map(|col| {
                let self_field = &col.ident_column;
                let target_field: Ident = syn::Ident::new(&col.column_name, proc_macro2::Span::call_site());
                quote! {
                    #target_field: auto_morphic_core::morph::Morph::morph_to(&self.#self_field),
                }
            })
            .collect();

        let flatten_from_result_fields: Vec<proc_macro2::TokenStream> = columns
            .iter()
            .filter(|col| col.flatten_from)
            .map(|col| {
                let self_field = &col.ident_column;
                let source_field: Ident = syn::Ident::new(&col.column_name, proc_macro2::Span::call_site());
                quote! {
                    #self_field: auto_morphic_core::morph::Morph::morph_from(t.#source_field),
                }
            })
            .collect();

        let morph_to_result_body = {
            let all_to_result_fields: Vec<proc_macro2::TokenStream> = morph_to_result_fields
                .into_iter()
                .chain(flatten_to_result_fields)
                .collect();
            if has_skip_to {
                quote! {
                    #(#all_to_result_fields)*
                    ..Default::default()
                }
            } else {
                quote! {
                    #(#all_to_result_fields)*
                }
            }
        };

        let morph_from_result_body = {
            let all_from_result_fields: Vec<proc_macro2::TokenStream> = morph_from_result_fields
                .into_iter()
                .chain(flatten_from_result_fields)
                .collect();
            if has_skip_from {
                quote! {
                    #(#all_from_result_fields)*
                    ..Default::default()
                }
            } else {
                quote! {
                    #(#all_from_result_fields)*
                }
            }
        };

        // Generate MorphResult impl blocks per target
        let result_impl_blocks = map_to_structs.iter().map(|target| {
            quote! {
                impl #impl_generics auto_morphic_core::morph::MorphResult<#target> for #type_name #ty_generics #enhanced_where_clause {
                    fn morph_to_result(&self) -> Result<#target, auto_morphic_core::morph::MorphError> {
                        Ok(#target {
                            #morph_to_result_body
                        })
                    }
                    fn morph_from_result(t: #target) -> Result<Self, auto_morphic_core::morph::MorphError> {
                        Ok(Self {
                            #morph_from_result_body
                        })
                    }
                }
            }
        });

        quote! {
            #(#result_impl_blocks)*
        }
    } else {
        quote! {}
    };

    // Generate impl blocks per target
    let impl_blocks = map_to_structs.iter().map(|target| {
        quote! {
            impl #impl_generics auto_morphic_core::morph::Morph<#target> for #type_name #ty_generics #enhanced_where_clause {
                fn morph_to(&self) -> #target {
                    #target {
                        #morph_to_body
                    }
                }
                fn morph_from(t: #target) -> Self {
                    Self {
                        #morph_from_body
                    }
                }
            }
        }
    });

    Ok(quote! {
        #(#impl_blocks)*
        #morph_result_impls
    })
}

/// Generate Morph impl blocks for an enum (unit variants with matching names)
fn generate_enum_impls(
    input: &DeriveInput,
    type_name: &Ident,
    variants: &[&Variant],
    map_to_structs: &[Path],
    impl_generics: &syn::ImplGenerics,
    ty_generics: &syn::TypeGenerics,
    where_clause: Option<&syn::WhereClause>,
) -> syn::Result<proc_macro2::TokenStream> {
    if map_to_structs.is_empty() {
        return Err(syn::Error::new_spanned(
            type_name,
            "AutoMorph requires at least one #[map_to(TargetEnum)] attribute on the enum\n\
             Example:\n  #[derive(AutoMorph)]\n  #[map_to(TargetEnum)]\n  enum MyEnum { A, B }",
        ));
    }

    // Parse variant mappings: #[map(name = "TargetVariantName")]
    struct EnumVariantInfo {
        ident: Ident,
        target_name: String,
    }

    let variant_infos: Vec<EnumVariantInfo> = variants
        .iter()
        .map(|v| {
            let mut target_name = v.ident.to_string();
            for attr in &v.attrs {
                if attr.path().is_ident("map")
                    && let Ok(meta_list) = attr.parse_args_with(syn::punctuated::Punctuated::<Meta, Token![,]>::parse_terminated)
                {
                    for meta in meta_list {
                        if let Meta::NameValue(nv) = meta
                            && nv.path.is_ident("name")
                            && let syn::Expr::Lit(syn::ExprLit { lit: syn::Lit::Str(lit_str), .. }) = &nv.value
                        {
                            target_name = lit_str.value();
                        }
                    }
                }
            }
            Ok(EnumVariantInfo {
                ident: v.ident.clone(),
                target_name,
            })
        })
        .collect::<syn::Result<Vec<_>>>()?;

    // Check all variants are unit variants (no data)
    for v in variants {
        match &v.fields {
            Fields::Unit => {} // OK
            _ => {
                return Err(syn::Error::new_spanned(
                    *v,
                    "AutoMorph enum derive currently only supports unit variants (no data). For data-carrying variants, use manual Morph implementation.",
                ));
            }
        }
    }

    let type_param_names = collect_type_param_names(&input.generics);
    let type_param_ids: Vec<Ident> = type_param_names
        .iter()
        .map(|name| syn::Ident::new(name, proc_macro2::Span::call_site()))
        .collect();

    let enhanced_where_clause = if !type_param_ids.is_empty() {
        let clone_bounds: Vec<proc_macro2::TokenStream> = type_param_ids.iter().map(|id| {
            quote! { #id: Clone }
        }).collect();

        if let Some(existing_where) = where_clause {
            quote! {
                #existing_where #(#clone_bounds,)*
            }
        } else {
            quote! {
                where #(#clone_bounds),*
            }
        }
    } else {
        quote! {}
    };

    // Build impl blocks per target — arms must be built inside the closure so `target` is in scope
    let impl_blocks = map_to_structs.iter().map(|target| {
        // Build morph_to match arms
        let morph_to_arms = variant_infos.iter().map(|vi| {
            let src_var = &vi.ident;
            let tgt_var: Ident = syn::Ident::new(&vi.target_name, proc_macro2::Span::call_site());
            quote! {
                Self::#src_var => #target::#tgt_var,
            }
        });

        // Build morph_from match arms
        let morph_from_arms = variant_infos.iter().map(|vi| {
            let tgt_var: Ident = syn::Ident::new(&vi.target_name, proc_macro2::Span::call_site());
            let src_var = &vi.ident;
            quote! {
                #target::#tgt_var => Self::#src_var,
            }
        });

        quote! {
            impl #impl_generics auto_morphic_core::morph::Morph<#target> for #type_name #ty_generics #enhanced_where_clause {
                fn morph_to(&self) -> #target {
                    match self {
                        #(#morph_to_arms)*
                    }
                }
                fn morph_from(t: #target) -> Self {
                    match t {
                        #(#morph_from_arms)*
                    }
                }
            }
        }
    });

    Ok(quote! {
        #(#impl_blocks)*
    })
}