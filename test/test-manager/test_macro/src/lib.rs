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
/// * `must_succeed` - If the testing suite stops running if this test fails. `must_succeed`
///   defaults to false.
///
/// * `always_run` - If the test should always run regardless of what test filters are provided by
///   the user. `always_run` defaults to false.
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
/// This test will run early in the test loop and must succeed and will always run.
///
/// ```ignore
/// #[test_function(priority = -1337, must_succeed = true, always_run = true)]
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
    let function_parameters = get_test_function_parameters(&function.sig.inputs)?;

    Ok(TestFunction {
        name: function.sig.ident.clone(),
        function_parameters,
        macro_parameters,
    })
}

fn get_test_macro_parameters(attributes: &syn::AttributeArgs) -> Result<MacroParameters> {
    let mut priority = None;
    let mut always_run = false;
    let mut must_succeed = false;
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
        } else if nv.path.is_ident("always_run") {
            match lit {
                Lit::Bool(lit_bool) => always_run = lit_bool.value(),
                _ => bail!(nv, "'always_run' should have a bool value"),
            }
        } else if nv.path.is_ident("must_succeed") {
            match lit {
                Lit::Bool(lit_bool) => must_succeed = lit_bool.value(),
                _ => bail!(nv, "'must_succeed' should have a bool value"),
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

    Ok(MacroParameters {
        priority,
        always_run,
        must_succeed,
        targets,
    })
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

    let always_run = test_function.macro_parameters.always_run;
    let must_succeed = test_function.macro_parameters.must_succeed;

    let func_name = test_function.name;
    let function_mullvad_version = test_function.function_parameters.mullvad_client.version();
    let wrapper_closure = match test_function.function_parameters.mullvad_client {
        MullvadClient::New { .. } => {
            quote! {
                |test_context: crate::tests::TestContext,
                rpc: test_rpc::ServiceClient,
                mullvad_client: crate::mullvad_daemon::MullvadClientArgument|
                {
                    let mullvad_client = match mullvad_client {
                        crate::mullvad_daemon::MullvadClientArgument::WithClient(client) => client,
                        crate::mullvad_daemon::MullvadClientArgument::None => unreachable!("invalid mullvad client")
                    };
                    Box::pin(async move {
                        #func_name(test_context, rpc, mullvad_client).await.map_err(Into::into)
                    })
                }
            }
        }
        MullvadClient::None { .. } => {
            quote! {
                |test_context: crate::tests::TestContext,
                rpc: test_rpc::ServiceClient,
                _mullvad_client: crate::mullvad_daemon::MullvadClientArgument| {
                    Box::pin(async move {
                        #func_name(test_context, rpc).await.map_err(Into::into)
                    })
                }
            }
        }
    };

    quote! {
        inventory::submit!(crate::tests::test_metadata::TestMetadata {
            name: stringify!(#func_name),
            command: stringify!(#func_name),
            targets: &[#targets],
            mullvad_client_version: #function_mullvad_version,
            func: #wrapper_closure,
            priority: #test_function_priority,
            always_run: #always_run,
            must_succeed: #must_succeed,
        });
    }
}

struct TestFunction {
    name: syn::Ident,
    function_parameters: FunctionParameters,
    macro_parameters: MacroParameters,
}

struct MacroParameters {
    priority: Option<i32>,
    always_run: bool,
    must_succeed: bool,
    targets: Vec<Os>,
}

enum MullvadClient {
    None {
        mullvad_client_version: proc_macro2::TokenStream,
    },
    New {
        mullvad_client_version: proc_macro2::TokenStream,
    },
}

impl MullvadClient {
    fn version(&self) -> proc_macro2::TokenStream {
        match self {
            MullvadClient::None {
                mullvad_client_version,
            } => mullvad_client_version.clone(),
            MullvadClient::New {
                mullvad_client_version,
                ..
            } => mullvad_client_version.clone(),
        }
    }
}

struct FunctionParameters {
    mullvad_client: MullvadClient,
}

fn get_test_function_parameters(
    args: &syn::punctuated::Punctuated<syn::FnArg, syn::Token![,]>,
) -> Result<FunctionParameters> {
    if args.len() <= 2 {
        return Ok(FunctionParameters {
            mullvad_client: MullvadClient::None {
                mullvad_client_version: quote! {
                    test_rpc::mullvad_daemon::MullvadClientVersion::None
                },
            },
        });
    }

    let arg = args[2].clone();
    let syn::FnArg::Typed(pat_type) = arg else {
        bail!(arg, "unexpected 'mullvad_client' arg");
    };

    let syn::Type::Path(syn::TypePath { path, .. }) = &*pat_type.ty else {
        bail!(pat_type, "unexpected 'mullvad_client' type");
    };

    let mullvad_client = match path.segments[0].ident.to_string().as_str() {
        "mullvad_management_interface" | "MullvadProxyClient" => {
            let mullvad_client_version =
                quote! { test_rpc::mullvad_daemon::MullvadClientVersion::New };
            MullvadClient::New {
                mullvad_client_version,
            }
        }
        _ => bail!(pat_type, "cannot infer mullvad client type"),
    };

    Ok(FunctionParameters { mullvad_client })
}
