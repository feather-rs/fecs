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

    let (resources_init, world_ident) = find_function_parameters(sig.inputs.iter());

    let (world_ident, world_ty) = world_ident.unwrap_or((
        Ident::new("_world", Span::call_site()),
        quote! { &mut fecs::World },
    ));

    let content = &input.block;

    let sys_name = input.sig.ident.clone();

    let res = quote! {
        #[allow(non_camel_case_types)]
        #[derive(Clone)]
        pub struct #sys_name;

        impl fecs::RawSystem for #sys_name {
            fn run(&self, resources: &fecs::Resources, #world_ident: #world_ty, _executor: &fecs::Executor) {
                #(#resources_init)*
                #content
            }
        }

        fecs::inventory::submit!(fecs::SystemRegistration(std::sync::Mutex::new(Some(Box::new(#sys_name)))));
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

    let (_is_batch, event_ty) = match &*event_ty.ty {
        Type::Reference(r) => match *r.elem.clone() {
            Type::Slice(s) => (true, (&*s.elem).clone()),
            t => (false, t),
        },
        _ => unimplemented!(),
    };

    let (resources_init, world_ident) = find_function_parameters(sig.inputs.iter().skip(1));

    let (world_ident, world_ty) = world_ident.unwrap_or((
        Ident::new("_world", Span::call_site()),
        quote! { &mut fecs::World },
    ));

    let sys_name = input.sig.ident.clone();

    let content = &input.block;

    let res = quote! {
        #[allow(non_camel_case_types)]
        pub struct #sys_name;

        impl fecs::RawEventHandler for #sys_name {
            type Event = #event_ty;
            fn handle(&self, resources: &fecs::Resources, #world_ident: #world_ty, event: &#event_ty) {
                #(#resources_init)*

                #content
            }
        }
    };

    res.into()
}

fn find_function_parameters<'a>(
    inputs: impl Iterator<Item = &'a FnArg>,
) -> (Vec<TokenStream>, Option<(Ident, TokenStream)>) {
    // Vector of resource takes from the `Resources`.
    let mut resources_init = vec![];
    // Vector of resource variable names (`Ident`s).
    // Ident of the World variable.
    let mut world_ident = None;

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
            ArgType::World => world_ident = Some((ident, arg.ty.to_token_stream())),
            ArgType::Resource(res) => {
                let get_fn = if mutability.is_some() {
                    quote! { get_mut }
                } else {
                    quote! { get }
                };
                let init = quote! {
                    let #mutability #ident = resources.#get_fn::<#res>();
                    let #ident: &#mutability #res = &#mutability *#ident;
                };
                resources_init.push(init);
            }
        }
    }

    (resources_init, world_ident)
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

    let string = ty.ident.to_string();

    let ty = if &string == world {
        ArgType::World
    } else {
        let ty = &inner.path;
        ArgType::Resource(quote! { #ty })
    };

    (arg.mutability.clone(), ty)
}

enum ArgType {
    World,
    Resource(TokenStream),
}

fn arg(arg: &FnArg) -> &PatType {
    match arg {
        FnArg::Typed(ty) => ty,
        _ => panic!("systems may not accept `self` parameters"),
    }
}
