use jnix::jni::JNIEnv;
use jnix::jni::objects::{JObject, JString};
use jnix::jni::sys::{JNI_FALSE, JNI_TRUE, jboolean, jint};
use jnix::{FromJava, JnixEnv};
use mullvad_api::ApiEndpoint;
use mullvad_daemon::runtime::new_current_thread;
use std::path::Path;
use std::path::PathBuf;
use std::sync::OnceLock;
use talpid_error::ErrorExt;
use tokio::runtime::Runtime;

/// Problem report needs to work independently of the daemon - use a dedicated runtime.
fn runtime() -> &'static Runtime {
    static RT: OnceLock<Runtime> = OnceLock::new();
    RT.get_or_init(|| {
        new_current_thread()
            .build()
            .expect("failed to create tokio runtime")
    })
}

#[unsafe(no_mangle)]
#[expect(non_snake_case)]
pub extern "system" fn Java_net_mullvad_mullvadvpn_lib_repository_ProblemReportRepository_collectReport(
    env: JNIEnv<'_>,
    _: JObject<'_>,
    logDirectory: JString<'_>,
    extraAppLogsDirectory: JString<'_>,
    outputPath: JString<'_>,
    unverifiedPurchases: jint,
    pendingPurchases: jint,
) -> jboolean {
    let env = JnixEnv::from(env);
    let log_dir_string = String::from_java(&env, logDirectory);
    let log_dir = PathBuf::from(log_dir_string);
    let extra_logs_dir_string = String::from_java(&env, extraAppLogsDirectory);
    let extra_logs_dir = PathBuf::from(extra_logs_dir_string);
    let output_path = String::from_java(&env, outputPath);
    let unverified_purchases = i32::from_java(&env, unverifiedPurchases);
    let pending_purchases = i32::from_java(&env, pendingPurchases);

    let collector = mullvad_problem_report::ProblemReportCollector {
        extra_logs: vec![],
        redact_custom_strings: vec![],
        android_log_dir: log_dir,
        extra_logs_dir,
        unverified_purchases,
        pending_purchases,
    };

    match runtime().block_on(collector.write_to_path(&output_path)) {
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

#[unsafe(no_mangle)]
#[expect(non_snake_case)]
pub extern "system" fn Java_net_mullvad_mullvadvpn_lib_repository_ProblemReportRepository_sendProblemReport(
    env: JNIEnv<'_>,
    _: JObject<'_>,
    userEmail: JString<'_>,
    userMessage: JString<'_>,
    accountToken: JString<'_>,
    outputPath: JString<'_>,
    cacheDirectory: JString<'_>,
    endpoint: JObject<'_>,
) -> jboolean {
    let env = JnixEnv::from(env);
    let user_email = String::from_java(&env, userEmail);
    let user_message = String::from_java(&env, userMessage);
    let account_token = if accountToken.is_null() {
        None
    } else {
        Some(String::from_java(&env, accountToken))
    };
    let output_path_string = String::from_java(&env, outputPath);
    let output_path = Path::new(&output_path_string);
    let cache_directory_string = String::from_java(&env, cacheDirectory);
    let cache_directory = Path::new(&cache_directory_string);
    let api_endpoint =
        crate::api::api_endpoint_from_java(&env, endpoint).unwrap_or(ApiEndpoint::from_env_vars());

    let send = mullvad_problem_report::send_problem_report(
        &user_email,
        &user_message,
        account_token.as_deref(),
        output_path,
        cache_directory,
        api_endpoint,
    );

    match runtime().block_on(send) {
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
