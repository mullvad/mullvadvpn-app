use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{AttributeArgs, Lit, Meta, NestedMeta};

/// Register an `async` function to be run by `test-manager`.
///
/// The `test_function` macro will inject two arguments to your function:
///
/// * `rpc` - a [`test_rpc::client::ServiceClient]` used to make
/// remote-procedure calls inside the virtual machine running the test. This can
/// be used to perform arbitrary network requests, inspect the local file
/// system, rebooting ..
///
/// * `mullvad_client` - a
/// [`mullvad_management_interface::ManagementServiceClient`] which provides a
/// bi-directional communication channel with the `mullvad-daemon` running
/// inside of the virtual machine. All RPC-calls as defined by
/// [`mullvad_management_interface::client`] are available on `mullvad_client`.
///
///# Arguments
///
/// The `test_function` macro takes 4 optional arguments
///
/// * `priority` - The order in which tests will be run where low numbers run
/// before high numbers and tests with the same number run in undefined order.
/// `priority` defaults to 0.
///
/// * `cleanup` - If the cleanup function will run after the test is finished
/// and among other things reset the settings to the default value for the
/// daemon.
/// `cleanup` defaults to true.
///
/// * `must_succeed` - If the testing suite stops running if this test fails.
/// `must_succeed` defaults to false.
///
/// * `always_run` - If the test should always run regardless of what test
/// filters are provided by the user.
/// `always_run` defaults to false.
///
/// * `target_os` - The test should only run on the specified OS. This can currently be
/// set to `linux`, `windows`, or `macos`.
///
/// # Examples
///
/// ## Create a standard test.
///
/// Remember that [`test_function`] will inject `rpc` and `mullvad_client` for
/// us.
///
/// ```
/// #[test_function]
/// pub async fn test_function(
///     rpc: ServiceClient,
///     mut mullvad_client: mullvad_management_interface::ManagementServiceClient,
/// ) -> Result<(), Error> {
///     Ok(())
/// }
/// ```
///
/// ## Create a test with custom parameters
///
/// This test will run early in the test loop, won't perform any cleanup, must
/// succeed and will always run.
///
/// ```
/// #[test_function(priority = -1337, cleanup = false, must_succeed = true, always_run = true)]
/// pub async fn test_function(
///     rpc: ServiceClient,
///     mut mullvad_client: mullvad_management_interface::ManagementServiceClient,
/// ) -> Result<(), Error> {
///     Ok(())
/// }
/// ```
#[proc_macro_attribute]
pub fn test_function(attributes: TokenStream, code: TokenStream) -> TokenStream {
    let function: syn::ItemFn = syn::parse(code).unwrap();
    let attributes = syn::parse_macro_input!(attributes as AttributeArgs);

    let test_function = parse_marked_test_function(&attributes, &function);

    let register_test = create_test(test_function);

    quote! {
        #function
        #register_test
    }
    .into_token_stream()
    .into()
}

fn parse_marked_test_function(attributes: &AttributeArgs, function: &syn::ItemFn) -> TestFunction {
    let macro_parameters = get_test_macro_parameters(attributes);

    let function_parameters = get_test_function_parameters(&function.sig.inputs);

    TestFunction {
        name: function.sig.ident.clone(),
        function_parameters,
        macro_parameters,
    }
}

fn get_test_macro_parameters(attributes: &syn::AttributeArgs) -> MacroParameters {
    let mut priority = None;
    let mut cleanup = true;
    let mut always_run = false;
    let mut must_succeed = false;
    let mut target_os = None;

    for attribute in attributes {
        if let NestedMeta::Meta(Meta::NameValue(nv)) = attribute {
            if nv.path.is_ident("priority") {
                match &nv.lit {
                    Lit::Int(lit_int) => {
                        priority = Some(lit_int.base10_parse().unwrap());
                    }
                    _ => panic!("'priority' should have an integer value"),
                }
            } else if nv.path.is_ident("always_run") {
                match &nv.lit {
                    Lit::Bool(lit_bool) => {
                        always_run = lit_bool.value();
                    }
                    _ => panic!("'always_run' should have a bool value"),
                }
            } else if nv.path.is_ident("must_succeed") {
                match &nv.lit {
                    Lit::Bool(lit_bool) => {
                        must_succeed = lit_bool.value();
                    }
                    _ => panic!("'must_succeed' should have a bool value"),
                }
            } else if nv.path.is_ident("cleanup") {
                match &nv.lit {
                    Lit::Bool(lit_bool) => {
                        cleanup = lit_bool.value();
                    }
                    _ => panic!("'cleanup' should have a bool value"),
                }
            } else if nv.path.is_ident("target_os") {
                match &nv.lit {
                    Lit::Str(lit_str) => {
                        target_os = Some(lit_str.value());
                    }
                    _ => panic!("'target_os' should have a string value"),
                }
            }
        }
    }

    MacroParameters {
        priority,
        cleanup,
        always_run,
        must_succeed,
        target_os,
    }
}

fn create_test(test_function: TestFunction) -> proc_macro2::TokenStream {
    let test_function_priority = match test_function.macro_parameters.priority {
        Some(priority) => quote! { Some(#priority) },
        None => quote! { None },
    };
    let target_os = match test_function.macro_parameters.target_os {
        Some(target_os) => {
            quote! {
                Some(::std::str::FromStr::from_str(#target_os).expect("invalid target os"))
            }
        }
        None => quote! { None },
    };
    let should_cleanup = test_function.macro_parameters.cleanup;
    let always_run = test_function.macro_parameters.always_run;
    let must_succeed = test_function.macro_parameters.must_succeed;

    let func_name = test_function.name;
    let function_mullvad_version = test_function.function_parameters.mullvad_client.version();
    let wrapper_closure = match test_function.function_parameters.mullvad_client {
        MullvadClient::New {
            mullvad_client_type,
            ..
        } => {
            let mullvad_client_type = *mullvad_client_type;
            quote! {
                |test_context: crate::tests::TestContext,
                rpc: test_rpc::ServiceClient,
                mullvad_client: Box<dyn std::any::Any + Send>,|
                {
                    use std::any::Any;
                    let mullvad_client = mullvad_client.downcast::<#mullvad_client_type>().expect("invalid mullvad client");
                    Box::pin(async move {
                        #func_name(test_context, rpc, *mullvad_client).await
                    })
                }
            }
        }
        MullvadClient::None { .. } => {
            quote! {
                |test_context: crate::tests::TestContext,
                rpc: test_rpc::ServiceClient,
                mullvad_client: Box<dyn std::any::Any + Send>| {
                    Box::pin(async move {
                        #func_name(test_context, rpc).await
                    })
                }
            }
        }
    };

    quote! {
        inventory::submit!(crate::tests::test_metadata::TestMetadata {
            name: stringify!(#func_name),
            command: stringify!(#func_name),
            target_os: #target_os,
            mullvad_client_version: #function_mullvad_version,
            func: Box::new(#wrapper_closure),
            priority: #test_function_priority,
            always_run: #always_run,
            must_succeed: #must_succeed,
            cleanup: #should_cleanup,
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
    cleanup: bool,
    always_run: bool,
    must_succeed: bool,
    target_os: Option<String>,
}

enum MullvadClient {
    None {
        mullvad_client_version: proc_macro2::TokenStream,
    },
    New {
        mullvad_client_type: Box<syn::Type>,
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
    inputs: &syn::punctuated::Punctuated<syn::FnArg, syn::Token![,]>,
) -> FunctionParameters {
    if inputs.len() > 2 {
        match inputs[2].clone() {
            syn::FnArg::Typed(pat_type) => {
                let mullvad_client = match &*pat_type.ty {
                    syn::Type::Path(syn::TypePath { path, .. }) => {
                        match path.segments[0].ident.to_string().as_str() {
                            "mullvad_management_interface" | "ManagementServiceClient" => {
                                let mullvad_client_version =
                                    quote! { test_rpc::mullvad_daemon::MullvadClientVersion::New };
                                MullvadClient::New {
                                    mullvad_client_type: pat_type.ty,
                                    mullvad_client_version,
                                }
                            }
                            _ => panic!("cannot infer mullvad client type"),
                        }
                    }
                    _ => panic!("unexpected 'mullvad_client' type"),
                };
                FunctionParameters { mullvad_client }
            }
            syn::FnArg::Receiver(_) => panic!("unexpected 'mullvad_client' arg"),
        }
    } else {
        FunctionParameters {
            mullvad_client: MullvadClient::None {
                mullvad_client_version: quote! { test_rpc::mullvad_daemon::MullvadClientVersion::None },
            },
        }
    }
}
