// This file is part of Substrate.

// Copyright (C) Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use proc_macro2::TokenStream;
use quote::quote;
use syn::{
	punctuated::Punctuated, spanned::Spanned, Error, Expr, ExprLit, Lit, Meta, MetaNameValue,
	Result, Token,
};

fn parse_deprecated_meta(path: &TokenStream, attr: &syn::Attribute) -> Result<TokenStream> {
	match &attr.meta {
		Meta::List(meta_list) => {
			let parsed = meta_list
				.parse_args_with(Punctuated::<MetaNameValue, Token![,]>::parse_terminated)
				.map_err(|e| Error::new(attr.span(), e.to_string()))?;
			let (note, since) = parsed.iter().try_fold((None, None), |mut acc, item| {
				let value = match &item.value {
					Expr::Lit(ExprLit { lit: lit @ Lit::Str(_), .. }) => Ok(lit),
					_ => Err(Error::new(attr.span(), "Invalid deprecation attribute")),
				}?;
				if item.path.is_ident("note") {
					acc.0.replace(value);
				} else if item.path.is_ident("since") {
					acc.1.replace(value);
				} else {
				};
				Ok::<(Option<&syn::Lit>, Option<&syn::Lit>), Error>(acc)
			})?;
			note.map_or_else(
				|| Err(Error::new(attr.span(), "Invalid deprecation attribute: missing `note`")),
				|note| {
					let since = if let Some(str) = since {
						quote! { Some(#str) }
					} else {
						quote! { None }
					};
					let doc = quote! { #path::__private::metadata_ir::DeprecationStatus::Deprecated { note: #note, since: #since }};
					Ok(doc)
				},
			)
		},
		Meta::NameValue(MetaNameValue {
			value: Expr::Lit(ExprLit { lit: lit @ Lit::Str(_), .. }),
			..
		}) => {
			// #[deprecated = "lit"]
			let doc = quote! { #path::__private::metadata_ir::DeprecationStatus::Deprecated { note: #lit, since: None } };
			Ok(doc)
		},
		Meta::Path(_) => {
			// #[deprecated]
			Ok(quote! { #path::__private::metadata_ir::DeprecationStatus::DeprecatedWithoutNote })
		},
		_ => Err(Error::new(attr.span(), "Invalid deprecation attribute")),
	}
}

/// collects deprecation attribute if its present.
pub fn get_deprecation(path: &TokenStream, attrs: &[syn::Attribute]) -> Result<TokenStream> {
	attrs
		.iter()
		.find(|a| a.path().is_ident("deprecated"))
		.map(|a| parse_deprecated_meta(path, a))
		.unwrap_or_else(|| {
			Ok(quote! {#path::__private::metadata_ir::DeprecationStatus::NotDeprecated})
		})
}
