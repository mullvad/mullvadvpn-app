use crate::{
    logging::{Logger, Panic, TestOutput, TestResult},
    mullvad_daemon::{self, MullvadClientArgument, RpcClientProvider},
    summary::SummaryLogger,
    tests::{self, config::TEST_CONFIG, TestContext, TestMetadata},
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

struct TestHandler<'a> {
    rpc_provider: &'a RpcClientProvider,
    test_runner_client: &'a ServiceClient,
    failed_tests: Vec<&'static str>,
    successful_tests: Vec<&'static str>,
    summary_logger: Option<SummaryLogger>,
    print_failed_tests_only: bool,
    logger: Logger,
}

impl TestHandler<'_> {
    /// Run `tests::test_upgrade_app` and register the result
    async fn run_test<R, F>(
        &mut self,
        test: &F,
        test_name: &'static str,
        mullvad_client: MullvadClientArgument,
    ) -> Result<(), anyhow::Error>
    where
        F: Fn(super::tests::TestContext, ServiceClient, MullvadClientArgument) -> R,
        R: Future<Output = anyhow::Result<()>>,
    {
        log::info!("Running {test_name}");

        if self.print_failed_tests_only {
            // Stop live record
            self.logger.store_records(true);
        }

        let test_output = run_test_function(
            self.test_runner_client.clone(),
            mullvad_client,
            &test,
            test_name,
            TestContext {
                rpc_provider: self.rpc_provider.clone(),
            },
        )
        .await;

        if self.print_failed_tests_only {
            // Print results of failed test
            if test_output.result.failure() {
                self.logger.print_stored_records();
            } else {
                self.logger.flush_records();
            }
            self.logger.store_records(false);
        }

        test_output.print();

        register_test_result(
            test_output.result,
            &mut self.failed_tests,
            test_name,
            &mut self.successful_tests,
            self.summary_logger.as_mut(),
        )
        .await?;

        Ok(())
    }

    fn gather_results(self) -> TestResult {
        log::info!("TESTS THAT SUCCEEDED:");
        for test in self.successful_tests {
            log::info!("{test}");
        }

        log::info!("TESTS THAT FAILED:");
        for test in &self.failed_tests {
            log::info!("{test}");
        }

        if self.failed_tests.is_empty() {
            TestResult::Pass
        } else {
            TestResult::Fail(anyhow::anyhow!("Some tests failed"))
        }
    }
}

pub async fn run(
    instance: &dyn vm::VmInstance,
    tests: Vec<TestMetadata>,
    skip_wait: bool,
    print_failed_tests_only: bool,
    summary_logger: Option<SummaryLogger>,
) -> Result<TestResult> {
    log::trace!("Setting test constants");

    let pty_path = instance.get_pty();

    log::debug!("Connecting to {pty_path}");

    let serial_stream =
        tokio_serial::SerialStream::open(&tokio_serial::new(pty_path, BAUD)).unwrap();
    let (runner_transport, mullvad_daemon_transport, mut connection_handle, completion_handle) =
        test_rpc::transport::create_client_transports(serial_stream)?;

    if !skip_wait {
        connection_handle.wait_for_server().await?;
    }

    log::info!("Running client");

    let test_runner_client = ServiceClient::new(connection_handle.clone(), runner_transport);
    let rpc_provider = mullvad_daemon::new_rpc_client(connection_handle, mullvad_daemon_transport);

    print_os_version(&test_runner_client).await;

    let mut test_handler = TestHandler {
        rpc_provider: &rpc_provider,
        test_runner_client: &test_runner_client,
        failed_tests: vec![],
        successful_tests: vec![],
        summary_logger,
        print_failed_tests_only,
        logger: Logger::get_or_init(),
    };

    // We need to handle the upgrade test separately since it expects the daemon to *not* be
    // installed, which is done by `tests::prepare_daemon`, and only runs with
    // `app_package_to_upgrade_from_filename` set.
    // TODO: Extend `TestMetadata` and the `test_function` macro to specify what daemon state is
    // expected, and to allow for skipping tests on arbitrary conditions.
    if TEST_CONFIG.app_package_to_upgrade_from_filename.is_some() {
        test_handler
            .run_test(
                &tests::test_upgrade_app,
                "test_upgrade_app",
                MullvadClientArgument::None,
            )
            .await?;
    } else {
        log::warn!("No previous app to upgrade from, skipping upgrade test");
    };

    for test in tests {
        tests::prepare_daemon(&test_runner_client, &rpc_provider)
            .await
            .context("Failed to reset daemon before test")?;

        let mullvad_client = rpc_provider
            .mullvad_client(test.mullvad_client_version)
            .await;
        test_handler
            .run_test(&test.func, test.name, mullvad_client)
            .await?;
    }

    let result = test_handler.gather_results();

    // wait for cleanup
    drop(test_runner_client);
    drop(rpc_provider);
    let _ = tokio::time::timeout(Duration::from_secs(5), completion_handle).await;

    Ok(result)
}

async fn register_test_result(
    test_result: TestResult,
    failed_tests: &mut Vec<&str>,
    test_name: &'static str,
    successful_tests: &mut Vec<&str>,
    summary_logger: Option<&mut SummaryLogger>,
) -> anyhow::Result<()> {
    if let Some(logger) = summary_logger {
        logger
            .log_test_result(test_name, test_result.summary())
            .await
            .context("Failed to log test result")?
    };

    if test_result.failure() {
        failed_tests.push(test_name);
    } else {
        successful_tests.push(test_name);
    }

    Ok(())
}

pub async fn run_test_function<F, R>(
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
