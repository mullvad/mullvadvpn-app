use crate::{
    logging::{Panic, TestOutput, TestResult},
    mullvad_daemon::{self, MullvadClientArgument},
    summary::{self, maybe_log_test_result},
    tests::{self, config::TEST_CONFIG, get_tests, TestContext},
    vm,
};
use anyhow::{Context, Result};
use futures::FutureExt;
use std::{future::Future, panic, time::Duration};
use test_rpc::{logging::Output, ServiceClient};

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
) -> Result<TestResult> {
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

    let test_runner_client = ServiceClient::new(connection_handle.clone(), runner_transport);
    let mullvad_client_provider =
        mullvad_daemon::new_rpc_client(connection_handle, mullvad_daemon_transport);

    print_os_version(&test_runner_client).await;

    let mut tests = get_tests();

    tests.retain(|test| test.should_run_on_os(TEST_CONFIG.os));

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

    let test_context = TestContext {
        rpc_provider: mullvad_client_provider,
    };

    let mut successful_tests = vec![];
    let mut failed_tests = vec![];

    let logger = super::logging::Logger::get_or_init();

    if let Some(app_to_upgrade_from) = TEST_CONFIG.app_package_to_upgrade_from_filename.as_ref() {
        if let Err(e) =
            tests::test_upgrade_app(&test_context, &test_runner_client, app_to_upgrade_from).await
        {
            log::error!("Failed to to run 'test_upgrade_app': {e}");
        }
    } else {
        log::warn!("No previous app to upgrade from, skipping upgrade test");
    };

    for test in tests {
        crate::tests::prepare_daemon(&test_runner_client, &test_context.rpc_provider)
            .await
            .context("Failed to reset daemon before test")?;

        let mullvad_client = test_context
            .rpc_provider
            .mullvad_client(test.mullvad_client_version)
            .await;

        log::info!("Running {}", test.name);

        if print_failed_tests_only {
            // Stop live record
            logger.store_records(true);
        }

        // TODO: Log how long each test took to run.
        let test_output = run_test(
            test_runner_client.clone(),
            mullvad_client,
            &test.func,
            test.name,
            test_context.clone(),
        )
        .await;

        if print_failed_tests_only {
            // Print results of failed test
            if test_output.result.failure() {
                logger.print_stored_records();
            } else {
                logger.flush_records();
            }
            logger.store_records(false);
        }

        test_output.print();

        maybe_log_test_result(
            summary_logger.as_mut(),
            test.name,
            test_output.result.summary(),
        )
        .await
        .context("Failed to log test result")?;

        match test_output.result {
            TestResult::Panic(panic) => {
                failed_tests.push(test.name);
                final_result = TestResult::Panic(panic);
                if test.must_succeed {
                    break;
                }
            }
            TestResult::Fail(failure) => {
                failed_tests.push(test.name);
                final_result = TestResult::Fail(failure);
                if test.must_succeed {
                    break;
                }
            }
            TestResult::Pass => {
                successful_tests.push(test.name);
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

    Ok(final_result)
}

pub async fn run_test<F, R>(
    runner_rpc: ServiceClient,
    mullvad_rpc: MullvadClientArgument,
    test: &F,
    test_name: &'static str,
    test_context: super::tests::TestContext,
) -> TestOutput
where
    F: Fn(super::tests::TestContext, ServiceClient, MullvadClientArgument) -> R,
    R: Future<Output = anyhow::Result<()>>,
{
    let _flushed = runner_rpc.try_poll_output().await;

    // Assert that the test is unwind safe, this is the same assertion that cargo tests do. This
    // assertion being incorrect can not lead to memory unsafety however it could theoretically
    // lead to logic bugs. The problem of forcing the test to be unwind safe is that it causes a
    // large amount of unergonomic design.
    let result: TestResult =
        panic::AssertUnwindSafe(test(test_context, runner_rpc.clone(), mullvad_rpc))
            .catch_unwind()
            .await
            .map_err(Panic::new)
            .into();

    let mut output = vec![];
    if result.failure() {
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

async fn print_os_version(client: &ServiceClient) {
    match client.get_os_version().await {
        Ok(version) => {
            log::debug!("Guest OS version: {version}");
        }
        Err(error) => {
            log::debug!("Failed to obtain guest OS version: {error}");
        }
    }
}
