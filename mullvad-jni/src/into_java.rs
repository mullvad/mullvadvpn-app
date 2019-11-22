use jnix::{jni::objects::AutoLocal, JnixEnv};
use mullvad_types::{
    relay_constraints::Constraint, relay_list::RelayList, settings::Settings, states::TunnelState,
    version::AppVersionInfo, wireguard::KeygenEvent,
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
wrap_jnix_into_java!(TunConfig);
wrap_jnix_into_java!(RelayList);

wrap_jnix_into_java!(Constraint<T>
    where
        T: Clone + Eq + Debug + jnix::IntoJava<'borrow, 'env, JavaType = AutoLocal<'env, 'borrow>>
);

wrap_jnix_into_java!(KeygenEvent);
wrap_jnix_into_java!(Settings);
wrap_jnix_into_java!(TunnelState);
