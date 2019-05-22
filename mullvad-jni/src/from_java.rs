use crate::get_class;
use jni::{
    objects::{JObject, JString},
    JNIEnv,
};
use mullvad_types::relay_constraints::Constraint;
use std::{fmt::Debug, ops::Deref};

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

impl<'env, T> FromJava<'env> for Constraint<T>
where
    T: Clone + Debug + Eq + FromJava<'env>,
    T::JavaType: From<JObject<'env>>,
{
    type JavaType = JObject<'env>;

    fn from_java(env: &JNIEnv<'env>, source: Self::JavaType) -> Self {
        if is_instance_of(env, source, "net/mullvad/mullvadvpn/model/Constraint$Any") {
            Constraint::Any
        } else if is_instance_of(env, source, "net/mullvad/mullvadvpn/model/Constraint$Only") {
            let value = get_object_field(env, source, "value", "Ljava/lang/Object;");

            Constraint::Only(T::from_java(env, T::JavaType::from(value)))
        } else {
            panic!("Invalid Constraint Java sub-class");
        }
    }
}

fn is_instance_of<'env>(
    env: &JNIEnv<'env>,
    object: JObject<'env>,
    class_name: &'static str,
) -> bool {
    let class = get_class(class_name);

    env.is_instance_of(object, &class)
        .expect("Failed to check if an object is an instance of a specified class")
}

fn get_object_field<'env>(
    env: &JNIEnv<'env>,
    object: JObject<'env>,
    field_name: &str,
    field_type: &str,
) -> JObject<'env> {
    env.get_field(object, field_name, field_type)
        .expect("Failed to get field from object")
        .l()
        .expect("Field has incorrect Java type")
}
