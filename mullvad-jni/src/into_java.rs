use crate::get_class;
use jni::{
    objects::{JList, JObject, JString, JValue},
    sys::jint,
    JNIEnv,
};
use mullvad_types::{account::AccountData, settings::Settings};

pub trait IntoJava<'env> {
    type JavaType;

    fn into_java(self, env: &JNIEnv<'env>) -> Self::JavaType;
}

impl<'env, T> IntoJava<'env> for Option<T>
where
    T: IntoJava<'env>,
    T::JavaType: From<JObject<'env>>,
{
    type JavaType = T::JavaType;

    fn into_java(self, env: &JNIEnv<'env>) -> Self::JavaType {
        match self {
            Some(data) => data.into_java(env),
            None => T::JavaType::from(JObject::null()),
        }
    }
}

impl<'env> IntoJava<'env> for String {
    type JavaType = JString<'env>;

    fn into_java(self, env: &JNIEnv<'env>) -> Self::JavaType {
        env.new_string(&self).expect("Failed to create Java String")
    }
}

impl<'env, T> IntoJava<'env> for Vec<T>
where
    T: IntoJava<'env>,
    JObject<'env>: From<T::JavaType>,
{
    type JavaType = JObject<'env>;

    fn into_java(self, env: &JNIEnv<'env>) -> Self::JavaType {
        let class = get_class("java/util/ArrayList");
        let initial_capacity = self.len();
        let parameters = [JValue::Int(initial_capacity as jint)];

        let list_object = env
            .new_object(&class, "(I)V", &parameters)
            .expect("Failed to create ArrayList object");

        let list =
            JList::from_env(env, list_object).expect("Failed to create JList from ArrayList");

        for element in self {
            let java_element = env.auto_local(JObject::from(element.into_java(env)));

            list.add(java_element.as_obj())
                .expect("Failed to add element to ArrayList");
        }

        list_object
    }
}

impl<'env> IntoJava<'env> for AccountData {
    type JavaType = JObject<'env>;

    fn into_java(self, env: &JNIEnv<'env>) -> Self::JavaType {
        let class = get_class("net/mullvad/mullvadvpn/model/AccountData");
        let account_expiry = env.auto_local(JObject::from(self.expiry.to_string().into_java(env)));
        let parameters = [JValue::Object(account_expiry.as_obj())];

        env.new_object(&class, "(Ljava/lang/String;)V", &parameters)
            .expect("Failed to create AccountData Java object")
    }
}

impl<'env> IntoJava<'env> for Settings {
    type JavaType = JObject<'env>;

    fn into_java(self, env: &JNIEnv<'env>) -> Self::JavaType {
        let class = get_class("net/mullvad/mullvadvpn/model/Settings");
        let account_token = env.auto_local(JObject::from(self.get_account_token().into_java(env)));
        let parameters = [JValue::Object(account_token.as_obj())];

        env.new_object(&class, "(Ljava/lang/String;)V", &parameters)
            .expect("Failed to create Settings Java object")
    }
}
