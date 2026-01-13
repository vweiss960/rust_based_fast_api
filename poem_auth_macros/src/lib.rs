//! Procedural macros for poem_auth authorization
//!
//! This crate provides attribute macros to add zero-boilerplate authorization
//! checks to Poem handlers. Guards are automatically applied at the function
//! signature level, eliminating manual guard instantiation and error handling.
//!
//! # Examples
//!
//! ```ignore
//! use poem::{handler, Response};
//! use poem_auth::UserClaims;
//! use poem_auth_macros::require_group;
//!
//! #[require_group("admins")]
//! #[handler]
//! async fn admin_endpoint(claims: UserClaims) -> Response {
//!     // Only users with "admins" group can reach this code
//!     "Admin area".into()
//! }
//! ```

use proc_macro::TokenStream;
use quote::{quote, format_ident};
use syn::{
    parse_macro_input, ItemFn, LitStr, Token, parse::{Parse, ParseStream},
    FnArg, Pat, PatType,
};

/// Arguments parsed from macro attributes
struct GroupArgs {
    groups: Vec<String>,
}

impl Parse for GroupArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut groups = Vec::new();

        // Handle empty case (for single group macros with no parens)
        if input.is_empty() {
            return Ok(GroupArgs { groups });
        }

        loop {
            let lit: LitStr = input.parse()?;
            groups.push(lit.value());

            if input.is_empty() {
                break;
            }

            input.parse::<Token![,]>()?;

            // Allow trailing comma
            if input.is_empty() {
                break;
            }
        }

        Ok(GroupArgs { groups })
    }
}

/// Check if a function parameter is `claims: UserClaims`
fn has_claims_parameter(input: &ItemFn) -> bool {
    input
        .sig
        .inputs
        .iter()
        .any(|arg| {
            if let FnArg::Typed(PatType { pat, ty, .. }) = arg {
                if let Pat::Ident(pat_ident) = &**pat {
                    if pat_ident.ident == "claims" {
                        // Check if type is UserClaims
                        if let syn::Type::Path(type_path) = &**ty {
                            if let Some(last_segment) = type_path.path.segments.last() {
                                return last_segment.ident == "UserClaims";
                            }
                        }
                    }
                }
            }
            false
        })
}

/// Require a single group membership
///
/// Returns 403 Forbidden if the user doesn't have the specified group.
///
/// # Example
///
/// ```ignore
/// #[require_group("admins")]
/// #[handler]
/// async fn admin_panel(claims: UserClaims) -> Response {
///     "Welcome to admin panel".into()
/// }
/// ```
///
/// # Requirements
///
/// The handler must have a `claims: UserClaims` parameter. The handler
/// function must return a type that implements `IntoResponse`.
#[proc_macro_attribute]
pub fn require_group(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as GroupArgs);
    let mut item_fn = parse_macro_input!(input as ItemFn);

    if args.groups.is_empty() {
        return syn::Error::new_spanned(&item_fn.sig, "At least one group must be specified")
            .to_compile_error()
            .into();
    }

    if !has_claims_parameter(&item_fn) {
        return syn::Error::new_spanned(
            &item_fn.sig,
            "Handler must have a `claims: UserClaims` parameter to use authorization macros",
        )
        .to_compile_error()
        .into();
    }

    let group = &args.groups[0];
    let error_msg = format!("Forbidden: requires '{}' group", group);

    // Insert guard check at start of function body
    let original_block = item_fn.block.clone();
    let guard_check = quote! {
        let __guard = ::poem_auth::HasGroup(#group.to_string());
        if !__guard.check(&claims) {
            return (
                ::poem::http::StatusCode::FORBIDDEN,
                ::poem::web::Json(::serde_json::json!({
                    "error": #error_msg
                }))
            ).into_response();
        }
    };

    item_fn.block = Box::new(syn::parse_quote!({
        #guard_check
        #original_block
    }));

    quote!(#item_fn).into()
}

/// Require membership in ANY of the specified groups (OR logic)
///
/// Returns 403 Forbidden if the user doesn't have at least one of the groups.
///
/// # Example
///
/// ```ignore
/// #[require_any_groups("admins", "moderators")]
/// #[handler]
/// async fn moderation_panel(claims: UserClaims) -> Response {
///     "Moderation panel".into()
/// }
/// ```
///
/// # Requirements
///
/// The handler must have a `claims: UserClaims` parameter. The handler
/// function must return a type that implements `IntoResponse`.
#[proc_macro_attribute]
pub fn require_any_groups(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as GroupArgs);
    let mut item_fn = parse_macro_input!(input as ItemFn);

    if args.groups.is_empty() {
        return syn::Error::new_spanned(&item_fn.sig, "At least one group must be specified")
            .to_compile_error()
            .into();
    }

    if !has_claims_parameter(&item_fn) {
        return syn::Error::new_spanned(
            &item_fn.sig,
            "Handler must have a `claims: UserClaims` parameter to use authorization macros",
        )
        .to_compile_error()
        .into();
    }

    let groups_vec = args.groups.clone();
    let error_msg = if args.groups.len() == 1 {
        format!("Forbidden: requires '{}' group", args.groups[0])
    } else {
        format!("Forbidden: requires one of groups: {}", args.groups.join(", "))
    };

    // Insert guard check at start of function body
    let original_block = item_fn.block.clone();
    let guard_check = quote! {
        let __guard = ::poem_auth::HasAnyGroup(vec![#(#groups_vec.to_string()),*]);
        if !__guard.check(&claims) {
            return (
                ::poem::http::StatusCode::FORBIDDEN,
                ::poem::web::Json(::serde_json::json!({
                    "error": #error_msg
                }))
            ).into_response();
        }
    };

    item_fn.block = Box::new(syn::parse_quote!({
        #guard_check
        #original_block
    }));

    quote!(#item_fn).into()
}

/// Require membership in ALL of the specified groups (AND logic)
///
/// Returns 403 Forbidden if the user doesn't have all of the groups.
///
/// # Example
///
/// ```ignore
/// #[require_all_groups("developers", "verified")]
/// #[handler]
/// async fn verified_dev_area(claims: UserClaims) -> Response {
///     "Verified developer area".into()
/// }
/// ```
///
/// # Requirements
///
/// The handler must have a `claims: UserClaims` parameter. The handler
/// function must return a type that implements `IntoResponse`.
#[proc_macro_attribute]
pub fn require_all_groups(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as GroupArgs);
    let mut item_fn = parse_macro_input!(input as ItemFn);

    if args.groups.is_empty() {
        return syn::Error::new_spanned(&item_fn.sig, "At least one group must be specified")
            .to_compile_error()
            .into();
    }

    if !has_claims_parameter(&item_fn) {
        return syn::Error::new_spanned(
            &item_fn.sig,
            "Handler must have a `claims: UserClaims` parameter to use authorization macros",
        )
        .to_compile_error()
        .into();
    }

    let groups_vec = args.groups.clone();
    let error_msg = format!("Forbidden: requires all groups: {}", args.groups.join(", "));

    // Insert guard check at start of function body
    let original_block = item_fn.block.clone();
    let guard_check = quote! {
        let __guard = ::poem_auth::HasAllGroups(vec![#(#groups_vec.to_string()),*]);
        if !__guard.check(&claims) {
            return (
                ::poem::http::StatusCode::FORBIDDEN,
                ::poem::web::Json(::serde_json::json!({
                    "error": #error_msg
                }))
            ).into_response();
        }
    };

    item_fn.block = Box::new(syn::parse_quote!({
        #guard_check
        #original_block
    }));

    quote!(#item_fn).into()
}
