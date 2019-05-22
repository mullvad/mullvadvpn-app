use jni::{
    objects::{JObject, JString},
    JNIEnv,
};
use std::ops::Deref;

pub trait FromJava<'env> {
    type JavaType: 'env;

    fn from_java(env: &JNIEnv<'env>, source: Self::JavaType) -> Self;
}

impl<'env, T> FromJava<'env> for Option<T>
where
    T: FromJava<'env>,
    T::JavaType: Deref<Target = JObject<'env>>,
{
    type JavaType = T::JavaType;

    fn from_java(env: &JNIEnv<'env>, source: Self::JavaType) -> Self {
        if source.is_null() {
            None
        } else {
            Some(T::from_java(env, source))
        }
    }
}

impl<'env> FromJava<'env> for String {
    type JavaType = JString<'env>;

    fn from_java(env: &JNIEnv<'env>, source: Self::JavaType) -> Self {
        String::from(
            env.get_string(source)
                .expect("Failed to convert from Java String"),
        )
    }
}
