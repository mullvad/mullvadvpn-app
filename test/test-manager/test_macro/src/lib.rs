use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{AttributeArgs, Lit, Meta, NestedMeta, Result};
use test_rpc::meta::Os;

/// Register an `async` function to be run by `test-manager`.
///
/// The `test_function` macro will inject two arguments to your function:
///
/// * `rpc` - a [`test_rpc::client::ServiceClient]` used to make remote-procedure calls inside the
///   virtual machine running the test. This can be used to perform arbitrary network requests,
///   inspect the local file system, rebooting ..
///
/// * `mullvad_client` - a [`mullvad_management_interface::MullvadProxyClient`] which provides a
///   bi-directional communication channel with the `mullvad-daemon` running inside of the virtual
///   machine. All RPC-calls as defined in [`mullvad_management_interface::MullvadProxyClient`] are
///   available on `mullvad_client`.
///
/// # Arguments
///
/// The `test_function` macro takes 4 optional arguments
///
/// * `priority` - The order in which tests will be run where low numbers run before high numbers
///   and tests with the same number run in undefined order. `priority` defaults to 0.
///
/// * `target_os` - The test should only run on the specified OS. This can currently be set to
///   `linux`, `windows`, or `macos`.
///
/// # Examples
///
/// ## Create a standard test.
///
/// Remember that [`test_function`] will inject `rpc` and `mullvad_client` for
/// us.
///
/// ```ignore
/// #[test_function]
/// pub async fn test_function(
///     rpc: ServiceClient,
///     mut mullvad_client: mullvad_management_interface::MullvadProxyClient,
/// ) -> anyhow::Result<()> {
///     Ok(())
/// }
/// ```
///
/// ## Create a test with custom parameters
///
/// This test will run early in the test loop.
///
/// ```ignore
/// #[test_function(priority = -1337)]
/// pub async fn test_function(
///     rpc: ServiceClient,
///     mut mullvad_client: mullvad_management_interface::MullvadProxyClient,
/// ) -> anyhow::Result<()> {
///     Ok(())
/// }
/// ```
#[proc_macro_attribute]
pub fn test_function(attributes: TokenStream, code: TokenStream) -> TokenStream {
    let function: syn::ItemFn = syn::parse(code).unwrap();
    let attributes = syn::parse_macro_input!(attributes as AttributeArgs);

    let test_function = match parse_marked_test_function(&attributes, &function) {
        Ok(tf) => tf,
        Err(e) => return e.into_compile_error().into(),
    };

    let register_test = create_test(test_function);

    quote! {
        #function
        #register_test
    }
    .into_token_stream()
    .into()
}

/// Shorthand for `return syn::Error::new(...)`.
macro_rules! bail {
    ($span:expr, $($tt:tt)*) => {{
        return ::core::result::Result::Err(::syn::Error::new(
            ::syn::spanned::Spanned::span(&$span),
            ::core::format_args!($($tt)*),
        ))
    }};
}

fn parse_marked_test_function(
    attributes: &AttributeArgs,
    function: &syn::ItemFn,
) -> Result<TestFunction> {
    let macro_parameters = get_test_macro_parameters(attributes)?;

    Ok(TestFunction {
        name: function.sig.ident.clone(),
        macro_parameters,
    })
}

fn get_test_macro_parameters(attributes: &syn::AttributeArgs) -> Result<MacroParameters> {
    let mut priority = None;
    let mut targets = vec![];

    for attribute in attributes {
        // we only use name-value attributes
        let NestedMeta::Meta(Meta::NameValue(nv)) = attribute else {
            bail!(attribute, "unknown attribute");
        };
        let lit = &nv.lit;

        if nv.path.is_ident("priority") {
            match lit {
                Lit::Int(lit_int) => priority = Some(lit_int.base10_parse().unwrap()),
                _ => bail!(nv, "'priority' should have an integer value"),
            }
        } else if nv.path.is_ident("target_os") {
            let Lit::Str(lit_str) = lit else {
                bail!(nv, "'target_os' should have a string value");
            };

            let target = match lit_str.value().parse() {
                Ok(os) => os,
                Err(e) => bail!(lit_str, "{e}"),
            };

            if targets.contains(&target) {
                bail!(nv, "Duplicate target");
            }

            targets.push(target);
        } else {
            bail!(nv, "unknown attribute");
        }
    }

    Ok(MacroParameters { priority, targets })
}

fn create_test(test_function: TestFunction) -> proc_macro2::TokenStream {
    let test_function_priority = match test_function.macro_parameters.priority {
        Some(priority) => quote! { Some(#priority) },
        None => quote! { None },
    };
    let targets: proc_macro2::TokenStream = (test_function.macro_parameters.targets.iter())
        .map(|&os| match os {
            Os::Linux => quote! { ::test_rpc::meta::Os::Linux, },
            Os::Macos => quote! { ::test_rpc::meta::Os::Macos, },
            Os::Windows => quote! { ::test_rpc::meta::Os::Windows, },
        })
        .collect();

    let func_name = test_function.name;
    let wrapper_closure = quote! {
        |test_context: crate::tests::TestContext,
        rpc: test_rpc::ServiceClient,
        mullvad_client: Option<MullvadProxyClient>|
        {
            let mullvad_client = match mullvad_client {
                Some(client) => client,
                None => unreachable!("invalid mullvad client")
            };
            Box::pin(async move {
                #func_name(test_context, rpc, mullvad_client).await.map_err(Into::into)
            })
        }
    };

    quote! {
        inventory::submit!(crate::tests::test_metadata::TestMetadata {
            name: stringify!(#func_name),
            targets: &[#targets],
            func: #wrapper_closure,
            priority: #test_function_priority,
            location: None,
        });
    }
}

struct TestFunction {
    name: syn::Ident,
    macro_parameters: MacroParameters,
}

struct MacroParameters {
    priority: Option<i32>,
    targets: Vec<Os>,
}
