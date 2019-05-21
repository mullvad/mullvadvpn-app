use jni::{objects::JString, JNIEnv};

pub trait FromJava<'env> {
    type JavaType: 'env;

    fn from_java(env: &JNIEnv<'env>, source: Self::JavaType) -> Self;
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
