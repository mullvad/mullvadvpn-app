use crate::daemon_interface;
use jnix::{
    jni::objects::{AutoLocal, JValue},
    JnixEnv,
};
use mullvad_types::{
    account::AccountData, relay_constraints::Constraint, relay_list::RelayList, settings::Settings,
    states::TunnelState, version::AppVersionInfo, wireguard::KeygenEvent,
};
use std::fmt::Debug;
use talpid_core::tunnel::tun_provider::TunConfig;

pub trait IntoJava<'borrow, 'env: 'borrow> {
    type JavaType;

    fn into_java(self, env: &'borrow JnixEnv<'env>) -> Self::JavaType;
}

macro_rules! wrap_jnix_into_java {
    ( $type:ty $( where $param:ident : $( $constraints:tt )* )* ) => {
        impl<'borrow, 'env, $( $param ),* > IntoJava<'borrow, 'env> for $type
        where
            'env: 'borrow,
            $( $param: $( $constraints )* ),*
        {
            type JavaType = <$type as jnix::IntoJava<'borrow, 'env>>::JavaType;

            fn into_java(self, env: &'borrow JnixEnv<'env>) -> Self::JavaType {
                jnix::IntoJava::into_java(self, env)
            }
        }
    };
}

wrap_jnix_into_java!(
    Option<T> where T: jnix::IntoJava<'borrow, 'env, JavaType = AutoLocal<'env, 'borrow>>
);

wrap_jnix_into_java!(String);

wrap_jnix_into_java!(
    Vec<T> where T: jnix::IntoJava<'borrow, 'env, JavaType = AutoLocal<'env, 'borrow>>
);

impl<'array, 'borrow, 'env> IntoJava<'borrow, 'env> for &'array [u8]
where
    'env: 'borrow,
{
    type JavaType = AutoLocal<'env, 'borrow>;

    fn into_java(self, env: &'borrow JnixEnv<'env>) -> Self::JavaType {
        jnix::IntoJava::into_java(self, env)
    }
}

wrap_jnix_into_java!(AppVersionInfo);
wrap_jnix_into_java!(AccountData);
wrap_jnix_into_java!(TunConfig);
wrap_jnix_into_java!(RelayList);

wrap_jnix_into_java!(Constraint<T>
    where
        T: Clone + Eq + Debug + jnix::IntoJava<'borrow, 'env, JavaType = AutoLocal<'env, 'borrow>>
);

wrap_jnix_into_java!(KeygenEvent);
wrap_jnix_into_java!(Settings);
wrap_jnix_into_java!(TunnelState);

impl<'borrow, 'env> IntoJava<'borrow, 'env> for Result<AccountData, daemon_interface::Error>
where
    'env: 'borrow,
{
    type JavaType = AutoLocal<'env, 'borrow>;

    fn into_java(self, env: &'borrow JnixEnv<'env>) -> Self::JavaType {
        env.auto_local(match self {
            Ok(data) => {
                let class = env.get_class("net/mullvad/mullvadvpn/model/GetAccountDataResult$Ok");
                let java_account_data = data.into_java(&env);
                let parameters = [JValue::Object(java_account_data.as_obj())];

                env.new_object(
                    &class,
                    "(Lnet/mullvad/mullvadvpn/model/AccountData;)V",
                    &parameters,
                )
                .expect("Failed to create GetAccountDataResult.Ok Java object")
            }
            Err(error) => {
                let class_name = match error {
                    daemon_interface::Error::RpcError(jsonrpc_client_core::Error(
                        jsonrpc_client_core::ErrorKind::JsonRpcError(jsonrpc_core::Error {
                            code: jsonrpc_core::ErrorCode::ServerError(-200),
                            ..
                        }),
                        _,
                    )) => "net/mullvad/mullvadvpn/model/GetAccountDataResult$InvalidAccount",
                    daemon_interface::Error::RpcError(_) => {
                        "net/mullvad/mullvadvpn/model/GetAccountDataResult$RpcError"
                    }
                    _ => "net/mullvad/mullvadvpn/model/GetAccountDataResult$OtherError",
                };
                let class = env.get_class(class_name);

                env.new_object(&class, "()V", &[])
                    .expect("Failed to create a GetAccountDataResult error sub-class Java object")
            }
        })
    }
}
