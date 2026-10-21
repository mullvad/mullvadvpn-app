use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{
    FnArg, Ident, ItemFn, PatIdent, PatType, ReturnType, Type, punctuated::Punctuated, token::Comma,
};

#[proc_macro_attribute]
pub fn export_tokio(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut body = syn::parse_macro_input!(item as ItemFn);
    if body.sig.asyncness.is_none() {
        panic!("macro only works on async functions")
    }
    let sig_name_str = body.sig.ident.to_string();
    let args_used = body
        .sig
        .inputs
        .clone()
        .into_iter()
        .enumerate()
        .map(|(i, t)| match t {
            FnArg::Receiver(..) => Ident::new("self", Span::call_site()),
            _ => Ident::new(&format!("arg{i}"), Span::call_site()),
        })
        .collect::<Punctuated<_, Comma>>();

    let mut wrapper_sig = body.sig.clone();
    wrapper_sig.inputs = wrapper_sig
        .inputs
        .into_iter()
        .enumerate()
        .map(|(i, arg)| match arg {
            FnArg::Receiver(receiver) => FnArg::Receiver(receiver),
            FnArg::Typed(t) => FnArg::Typed(PatType {
                attrs: t.attrs,
                pat: Box::new(syn::Pat::Ident(PatIdent {
                    mutability: None,
                    attrs: Vec::new(),
                    by_ref: None,
                    subpat: None,
                    ident: Ident::new(&format!("arg{i}"), Span::call_site()),
                })),
                colon_token: t.colon_token,
                ty: t.ty,
            }),
        })
        .collect();

    let inner_ident = Ident::new("__inner__", Span::call_site());
    body.sig.ident = inner_ident.clone();

    if let ReturnType::Type(_, ty) = &mut wrapper_sig.output
        && let Type::Path(ty_path) = &mut **ty
        && let Some(first) = ty_path.path.segments.first_mut()
        && first.ident == "Result"
        && let syn::PathArguments::AngleBracketed(args) = &mut first.arguments
        && let Some(arg) = args.args.iter_mut().nth(1)
    {
        *arg = syn::parse2(quote! { crate::type_bridges::AnyError }).unwrap();
    }

    let wrapper = quote::quote! {
        #wrapper_sig {
            // we might not want this, test it
            #[track_caller]
            #body

            TokioToken::spawn(#inner_ident (#args_used))
                .await
                .map_err(AnyError::from) // we need to do this!
                .inspect_err(|e| log::error!("{} failed: {e}", #sig_name_str))
        }
    };

    wrapper.into()
}
