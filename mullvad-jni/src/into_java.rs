use crate::get_class;
use jni::{
    objects::{JList, JObject, JString, JValue},
    sys::jint,
    JNIEnv,
};
use mullvad_types::{
    account::AccountData,
    relay_constraints::LocationConstraint,
    relay_list::{Relay, RelayList, RelayListCity, RelayListCountry},
    settings::Settings,
};

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

impl<'env> IntoJava<'env> for RelayList {
    type JavaType = JObject<'env>;

    fn into_java(self, env: &JNIEnv<'env>) -> Self::JavaType {
        let class = get_class("net/mullvad/mullvadvpn/model/RelayList");
        let relay_countries = env.auto_local(self.countries.into_java(env));
        let parameters = [JValue::Object(relay_countries.as_obj())];

        env.new_object(&class, "(Ljava/util/List;)V", &parameters)
            .expect("Failed to create RelayList Java object")
    }
}

impl<'env> IntoJava<'env> for RelayListCountry {
    type JavaType = JObject<'env>;

    fn into_java(self, env: &JNIEnv<'env>) -> Self::JavaType {
        let class = get_class("net/mullvad/mullvadvpn/model/RelayListCountry");
        let name = env.auto_local(JObject::from(self.name.into_java(env)));
        let code = env.auto_local(JObject::from(self.code.into_java(env)));
        let relay_cities = env.auto_local(self.cities.into_java(env));
        let parameters = [
            JValue::Object(name.as_obj()),
            JValue::Object(code.as_obj()),
            JValue::Object(relay_cities.as_obj()),
        ];

        env.new_object(
            &class,
            "(Ljava/lang/String;Ljava/lang/String;Ljava/util/List;)V",
            &parameters,
        )
        .expect("Failed to create RelayListCountry Java object")
    }
}

impl<'env> IntoJava<'env> for RelayListCity {
    type JavaType = JObject<'env>;

    fn into_java(self, env: &JNIEnv<'env>) -> Self::JavaType {
        let class = get_class("net/mullvad/mullvadvpn/model/RelayListCity");
        let name = env.auto_local(JObject::from(self.name.into_java(env)));
        let code = env.auto_local(JObject::from(self.code.into_java(env)));
        let relays = env.auto_local(self.relays.into_java(env));
        let parameters = [
            JValue::Object(name.as_obj()),
            JValue::Object(code.as_obj()),
            JValue::Object(relays.as_obj()),
        ];

        env.new_object(
            &class,
            "(Ljava/lang/String;Ljava/lang/String;Ljava/util/List;)V",
            &parameters,
        )
        .expect("Failed to create RelayListCity Java object")
    }
}

impl<'env> IntoJava<'env> for Relay {
    type JavaType = JObject<'env>;

    fn into_java(self, env: &JNIEnv<'env>) -> Self::JavaType {
        let class = get_class("net/mullvad/mullvadvpn/model/Relay");
        let hostname = env.auto_local(JObject::from(self.hostname.into_java(env)));
        let parameters = [JValue::Object(hostname.as_obj())];

        env.new_object(&class, "(Ljava/lang/String;)V", &parameters)
            .expect("Failed to create Relay Java object")
    }
}

impl<'env> IntoJava<'env> for LocationConstraint {
    type JavaType = JObject<'env>;

    fn into_java(self, env: &JNIEnv<'env>) -> Self::JavaType {
        match self {
            LocationConstraint::Country(country_code) => {
                let class = get_class("net/mullvad/mullvadvpn/model/LocationConstraint$Country");
                let country = env.auto_local(JObject::from(country_code.into_java(env)));
                let parameters = [JValue::Object(country.as_obj())];

                env.new_object(&class, "(Ljava/lang/String;)V", &parameters)
                    .expect("Failed to create LocationConstraint.Country Java object")
            }
            LocationConstraint::City(country_code, city_code) => {
                let class = get_class("net/mullvad/mullvadvpn/model/LocationConstraint$City");
                let country = env.auto_local(JObject::from(country_code.into_java(env)));
                let city = env.auto_local(JObject::from(city_code.into_java(env)));
                let parameters = [
                    JValue::Object(country.as_obj()),
                    JValue::Object(city.as_obj()),
                ];

                env.new_object(
                    &class,
                    "(Ljava/lang/String;Ljava/lang/String;)V",
                    &parameters,
                )
                .expect("Failed to create LocationConstraint.City Java object")
            }
            LocationConstraint::Hostname(country_code, city_code, hostname) => {
                let class = get_class("net/mullvad/mullvadvpn/model/LocationConstraint$Hostname");
                let country = env.auto_local(JObject::from(country_code.into_java(env)));
                let city = env.auto_local(JObject::from(city_code.into_java(env)));
                let hostname = env.auto_local(JObject::from(hostname.into_java(env)));
                let parameters = [
                    JValue::Object(country.as_obj()),
                    JValue::Object(city.as_obj()),
                    JValue::Object(hostname.as_obj()),
                ];

                env.new_object(
                    &class,
                    "(Ljava/lang/String;Ljava/lang/String;Ljava/lang/String;)V",
                    &parameters,
                )
                .expect("Failed to create LocationConstraint.Hostname Java object")
            }
        }
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
