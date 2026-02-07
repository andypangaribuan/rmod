/*
 * Copyright (c) 2026.
 * Created by Andy Pangaribuan (iam.pangaribuan@gmail.com)
 * https://github.com/apangaribuan
 *
 * This product is protected by copyright and distributed under
 * licenses restricting copying, distribution and decompilation.
 * All Rights Reserved.
 */

use proc_macro::TokenStream;
use quote::quote;
use syn::{ItemFn, parse_macro_input};

#[proc_macro_attribute]
pub fn fuse_handler(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);

    let vis = &input.vis;
    let sig = &input.sig;
    let body = &input.block;
    let name = &sig.ident;
    let attrs = &input.attrs;

    // We expect the signature to be: async fn name(ctx: &mut FuseRContext) -> FuseResult
    // We will transform it to: fn name(ctx: &mut FuseRContext) -> BoxFuture<'_, FuseResult>

    let expanded = quote! {
        #(#attrs)*
        #vis fn #name(ctx: &mut FuseRContext) -> rmod::fuse::BoxFuture<'_, FuseResult> {
            Box::pin(async move #body)
        }
    };

    TokenStream::from(expanded)
}
