use crate::summary::{self, maybe_log_test_result};
use crate::tests::{config::TEST_CONFIG, TestContext};
use crate::{
    logging::{panic_as_string, TestOutput},
    mullvad_daemon, tests,
    tests::Error,
    vm,
};
use anyhow::{Context, Result};
use futures::FutureExt;
use std::future::Future;
use std::panic;
use std::time::Duration;
use test_rpc::logging::Output;
use test_rpc::{mullvad_daemon::MullvadClientVersion, ServiceClient};

/// The baud rate of the serial connection between the test manager and the test runner.
/// There is a known issue with setting a baud rate at all or macOS, and the workaround
/// is to set it to zero: https://github.com/serialport/serialport-rs/pull/58
///
/// Keep this constant in sync with `test-runner/src/main.rs`
const BAUD: u32 = if cfg!(target_os = "macos") { 0 } else { 115200 };

pub async fn run(
    config: tests::config::TestConfig,
    instance: &dyn vm::VmInstance,
    test_filters: &[String],
    skip_wait: bool,
    print_failed_tests_only: bool,
    mut summary_logger: Option<summary::SummaryLogger>,
) -> Result<()> {
    log::trace!("Setting test constants");
    TEST_CONFIG.init(config);

    let pty_path = instance.get_pty();

    log::info!("Connecting to {pty_path}");

    let serial_stream =
        tokio_serial::SerialStream::open(&tokio_serial::new(pty_path, BAUD)).unwrap();
    let (runner_transport, mullvad_daemon_transport, mut connection_handle, completion_handle) =
        test_rpc::transport::create_client_transports(serial_stream)?;

    if !skip_wait {
        connection_handle.wait_for_server().await?;
    }

    log::info!("Running client");

    let client = ServiceClient::new(connection_handle.clone(), runner_transport);
    let mullvad_client =
        mullvad_daemon::new_rpc_client(connection_handle, mullvad_daemon_transport);

    let mut tests: Vec<_> = inventory::iter::<tests::TestMetadata>()
        .filter(|test| test.should_run_on_os(TEST_CONFIG.os))
        .collect();
    tests.sort_by_key(|test| test.priority.unwrap_or(0));

    if !test_filters.is_empty() {
        tests.retain(|test| {
            if test.always_run {
                return true;
            }
            for command in test_filters {
                let command = command.to_lowercase();
                if test.command.to_lowercase().contains(&command) {
                    return true;
                }
            }
            false
        });
    }

    let mut final_result = Ok(());

    let test_context = TestContext {
        rpc_provider: mullvad_client,
    };

    let mut successful_tests = vec![];
    let mut failed_tests = vec![];

    let logger = super::logging::Logger::get_or_init();

    for test in tests {
        let mclient = test_context
            .rpc_provider
            .as_type(test.mullvad_client_version)
            .await;

        log::info!("Running {}", test.name);

        if print_failed_tests_only {
            // Stop live record
            logger.store_records(true);
        }

        let test_result = run_test(
            client.clone(),
            mclient,
            &test.func,
            test.name,
            test_context.clone(),
        )
        .await;

        if test.mullvad_client_version == MullvadClientVersion::New {
            // Try to reset the daemon state if the test failed OR if the test doesn't explicitly
            // disabled cleanup.
            if test.cleanup || matches!(test_result.result, Err(_) | Ok(Err(_))) {
                let mut client = test_context.rpc_provider.new_client().await;
                crate::tests::cleanup_after_test(&mut client).await?;
            }
        }

        if print_failed_tests_only {
            // Print results of failed test
            if matches!(test_result.result, Err(_) | Ok(Err(_))) {
                logger.print_stored_records();
            } else {
                logger.flush_records();
            }
            logger.store_records(false);
        }

        test_result.print();

        let test_succeeded = matches!(test_result.result, Ok(Ok(_)));

        maybe_log_test_result(
            summary_logger.as_mut(),
            test.name,
            if test_succeeded {
                summary::TestResult::Pass
            } else {
                summary::TestResult::Fail
            },
        )
        .await
        .context("Failed to log test result")?;

        match test_result.result {
            Err(panic) => {
                failed_tests.push(test.name);
                final_result = Err(panic).context("test panicked");
                if test.must_succeed {
                    break;
                }
            }
            Ok(Err(failure)) => {
                failed_tests.push(test.name);
                final_result = Err(failure).context("test failed");
                if test.must_succeed {
                    break;
                }
            }
            Ok(Ok(result)) => {
                successful_tests.push(test.name);
                final_result = final_result.and(Ok(result));
            }
        }
    }

    log::info!("TESTS THAT SUCCEEDED:");
    for test in successful_tests {
        log::info!("{test}");
    }

    log::info!("TESTS THAT FAILED:");
    for test in failed_tests {
        log::info!("{test}");
    }

    // wait for cleanup
    drop(test_context);
    let _ = tokio::time::timeout(Duration::from_secs(5), completion_handle).await;

    final_result
}

pub async fn run_test<F, R, MullvadClient>(
    runner_rpc: ServiceClient,
    mullvad_rpc: MullvadClient,
    test: &F,
    test_name: &'static str,
    test_context: super::tests::TestContext,
) -> TestOutput
where
    F: Fn(super::tests::TestContext, ServiceClient, MullvadClient) -> R,
    R: Future<Output = Result<(), Error>>,
{
    let _flushed = runner_rpc.try_poll_output().await;

    // Assert that the test is unwind safe, this is the same assertion that cargo tests do. This
    // assertion being incorrect can not lead to memory unsafety however it could theoretically
    // lead to logic bugs. The problem of forcing the test to be unwind safe is that it causes a
    // large amount of unergonomic design.
    let result = panic::AssertUnwindSafe(test(test_context, runner_rpc.clone(), mullvad_rpc))
        .catch_unwind()
        .await
        .map_err(panic_as_string);

    let mut output = vec![];
    if matches!(result, Ok(Err(_)) | Err(_)) {
        let output_after_test = runner_rpc.try_poll_output().await;
        match output_after_test {
            Ok(mut output_after_test) => {
                output.append(&mut output_after_test);
            }
            Err(e) => {
                output.push(Output::Other(format!("could not get logs: {:?}", e)));
            }
        }
    }

    let log_output = runner_rpc.get_mullvad_app_logs().await.ok();

    TestOutput {
        log_output,
        test_name,
        error_messages: output,
        result,
    }
}
