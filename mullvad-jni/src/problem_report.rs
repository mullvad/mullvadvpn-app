use jnix::{
    jni::{
        objects::{JObject, JString},
        sys::{jboolean, JNI_FALSE, JNI_TRUE},
        JNIEnv,
    },
    FromJava, JnixEnv,
};
use std::path::Path;
use talpid_types::ErrorExt;

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn Java_net_mullvad_mullvadvpn_dataproxy_MullvadProblemReport_collectReport(
    env: JNIEnv<'_>,
    _: JObject<'_>,
    logDirectory: JString<'_>,
    outputPath: JString<'_>,
) -> jboolean {
    let env = JnixEnv::from(env);
    let log_dir_string = String::from_java(&env, logDirectory);
    let log_dir = Path::new(&log_dir_string);
    let output_path_string = String::from_java(&env, outputPath);
    let output_path = Path::new(&output_path_string);

    match mullvad_problem_report::collect_report(&[], output_path, Vec::new(), log_dir) {
        Ok(()) => JNI_TRUE,
        Err(error) => {
            log::error!(
                "{}",
                error.display_chain_with_msg("Failed to collect problem report")
            );
            JNI_FALSE
        }
    }
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn Java_net_mullvad_mullvadvpn_dataproxy_MullvadProblemReport_sendProblemReport(
    env: JNIEnv<'_>,
    _: JObject<'_>,
    userEmail: JString<'_>,
    userMessage: JString<'_>,
    outputPath: JString<'_>,
    resourcesDirectory: JString<'_>,
) -> jboolean {
    let env = JnixEnv::from(env);
    let user_email = String::from_java(&env, userEmail);
    let user_message = String::from_java(&env, userMessage);
    let output_path_string = String::from_java(&env, outputPath);
    let output_path = Path::new(&output_path_string);
    let resources_directory_string = String::from_java(&env, resourcesDirectory);
    let resources_directory = Path::new(&resources_directory_string);

    let send_result = mullvad_problem_report::send_problem_report(
        &user_email,
        &user_message,
        output_path,
        resources_directory,
    );

    match send_result {
        Ok(()) => JNI_TRUE,
        Err(error) => {
            log::error!(
                "{}",
                error.display_chain_with_msg("Failed to send problem report")
            );
            JNI_FALSE
        }
    }
}
