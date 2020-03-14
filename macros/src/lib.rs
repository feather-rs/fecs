extern crate proc_macro;

#[macro_use]
extern crate syn;
#[macro_use]
extern crate quote;

use proc_macro2::{Span, TokenStream};
use quote::ToTokens;
use syn::{FnArg, Ident, ItemFn, Pat, PatType, Type};

#[proc_macro_attribute]
pub fn system(
    _args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let input: ItemFn = parse_macro_input!(input as ItemFn);

    let sig = &input.sig;
    assert!(
        sig.generics.params.is_empty(),
        "systems may not have generic parameters"
    );

    let (resources_init, world_ident, ctx_ident) = find_function_parameters(sig.inputs.iter());

    let (world_ident, world_ty) = world_ident.unwrap_or((
        Ident::new("_world", Span::call_site()),
        quote! { &mut fecs::World },
    ));
    let (ctx_ident, ctx_ty) = ctx_ident.unwrap_or((
        Ident::new("_ctx", Span::call_site()),
        quote! { &mut fecs::SystemCtx },
    ));

    let content = &input.block;

    let sys_name = input.sig.ident.clone();

    let res = quote! {
        #[allow(non_camel_case_types)]
        pub struct #sys_name;

        impl fecs::RawSystem for #sys_name {
            fn run(&self, resources: &fecs::Resources, #world_ident: #world_ty, _executor: &fecs::Executor, #ctx_ident: #ctx_ty) {
                #(#resources_init)*
                #content
            }
        }
    };

    res.into()
}

#[proc_macro_attribute]
pub fn event_handler(
    _args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let input: ItemFn = parse_macro_input!(input as ItemFn);

    let sig = &input.sig;
    assert!(
        sig.generics.params.is_empty(),
        "systems may not have generic parameters"
    );

    // Find whether this is a batch handler or not, based on the first argument, which
    // is the event argument.
    let event_arg = sig
        .inputs
        .first()
        .expect("event handler must take event as its first parameter");
    let event_ty = match event_arg {
        FnArg::Typed(p) => p,
        _ => panic!("event handler may not take self parameter"),
    };

    let (is_batch, event_ty) = match &*event_ty.ty {
        Type::Reference(r) => match *r.elem.clone() {
            Type::Slice(s) => (true, (&*s.elem).clone()),
            t => (false, t),
        },
        _ => unimplemented!(),
    };

    let block_outer = if is_batch {
        quote! {}
    } else {
        quote! { for event in events }
    };

    let (resources_init, world_ident, ctx_ident) =
        find_function_parameters(sig.inputs.iter().skip(1));

    let (world_ident, world_ty) = world_ident.unwrap_or((
        Ident::new("_world", Span::call_site()),
        quote! { &mut fecs::World },
    ));
    let (ctx_ident, ctx_ty) = ctx_ident.unwrap_or((
        Ident::new("_ctx", Span::call_site()),
        quote! { &mut fecs::SystemCtx },
    ));

    let sys_name = input.sig.ident.clone();

    let content = &input.block;

    let res = quote! {
        #[allow(non_camel_case_types)]
        pub struct #sys_name;

        impl fecs::EventHandler<#event_ty> for #sys_name {
            fn handle(&self, events: &[#event_ty], resources: &fecs::Resources, #world_ident: #world_ty, _executor: &fecs::Executor, #ctx_ident: #ctx_ty) {
                #(#resources_init)*

                #block_outer {
                    #content
                }
            }
        }
    };

    res.into()
}

fn find_function_parameters<'a>(
    inputs: impl Iterator<Item = &'a FnArg>,
) -> (
    Vec<TokenStream>,
    Option<(Ident, TokenStream)>,
    Option<(Ident, TokenStream)>,
) {
    // Vector of resource takes from the `Resources`.
    let mut resources_init = vec![];
    // Vector of resource variable names (`Ident`s).
    // Ident of the World variable.
    let mut world_ident = None;
    // Ident of the SystemCtx variable.
    let mut ctx_ident = None;

    // Parse function arguments and determine whether they refer to resources,
    // the `PreparedWorld`, or the `CommandBuffer`.
    // Note that queries are performed inside the function using `cohort::query`.
    // This is implemented below.
    for param in inputs {
        let arg = arg(param);
        let ident = match &*arg.pat {
            Pat::Ident(ident) => ident.ident.clone(),
            _ => panic!(),
        };

        let (mutability, ty) = parse_arg(arg);

        match ty {
            ArgType::SystemCtx => ctx_ident = Some((ident, arg.ty.to_token_stream())),
            ArgType::World => world_ident = Some((ident, arg.ty.to_token_stream())),
            ArgType::Resource(res) => {
                let get_fn = if mutability.is_some() {
                    quote! { get_mut }
                } else {
                    quote! { get }
                };
                let init = quote! {
                    let #mutability #ident = resources.#get_fn::<#res>().unwrap();
                    let #ident: &#mutability #res = &#mutability *#ident;
                };
                resources_init.push(init);
            }
        }
    }

    (resources_init, world_ident, ctx_ident)
}

fn parse_arg(arg: &PatType) -> (Option<Token![mut]>, ArgType) {
    let arg = match &*arg.ty {
        Type::Reference(r) => r,
        _ => panic!("Invalid argument type"),
    };

    let inner = match &*arg.elem {
        Type::Path(path) => path,
        _ => panic!("Invalid argument type"),
    };

    let ty = inner.path.segments.last().expect("no last path segment");

    let world = "World";
    let ctx = "SystemCtx";

    let string = ty.ident.to_string();

    let ty = if &string == world {
        ArgType::World
    } else if &string == ctx {
        ArgType::SystemCtx
    } else {
        let ty = &inner.path;
        ArgType::Resource(quote! { #ty })
    };

    (arg.mutability.clone(), ty)
}

enum ArgType {
    World,
    SystemCtx,
    Resource(TokenStream),
}

fn arg(arg: &FnArg) -> &PatType {
    match arg {
        FnArg::Typed(ty) => ty,
        _ => panic!("systems may not accept `self` parameters"),
    }
}
