use anyhow::Error;
use colored::Colorize;
use std::sync::{Arc, Mutex};
use test_rpc::logging::{LogOutput, Output};

use crate::summary;

/// Logger that optionally supports logging records to a buffer
#[derive(Clone)]
pub struct Logger {
    inner: Arc<Mutex<LoggerInner>>,
}

struct LoggerInner {
    env_logger: env_logger::Logger,
    buffer: bool,
    stored_records: Vec<StoredRecord>,
}

struct StoredRecord {
    level: log::Level,
    time: chrono::DateTime<chrono::Local>,
    mod_path: String,
    text: String,
}

impl Logger {
    pub fn get_or_init() -> Self {
        static LOGGER: once_cell::sync::Lazy<Logger> = once_cell::sync::Lazy::new(|| {
            let mut logger = env_logger::Builder::new();
            logger.filter_module("h2", log::LevelFilter::Info);
            logger.filter_module("tower", log::LevelFilter::Info);
            logger.filter_module("hyper", log::LevelFilter::Info);
            logger.filter_module("rustls", log::LevelFilter::Info);
            logger.filter_module("tarpc", log::LevelFilter::Warn);
            logger.filter_module("mio_serial", log::LevelFilter::Warn);
            logger.filter_level(log::LevelFilter::Info);
            logger.parse_env(env_logger::DEFAULT_FILTER_ENV);

            let env_logger = logger.build();
            let max_level = env_logger.filter();

            let logger = Logger {
                inner: Arc::new(Mutex::new(LoggerInner {
                    env_logger,
                    buffer: false,
                    stored_records: vec![],
                })),
            };

            if log::set_boxed_logger(Box::new(logger.clone())).is_ok() {
                log::set_max_level(max_level);
            }

            logger
        });

        LOGGER.clone()
    }

    /// Set whether to buffer logs instead of printing them to stdout and stderr
    pub fn store_records(&self, state: bool) {
        let mut inner = self.inner.lock().unwrap();
        inner.buffer = state;
    }

    /// Flush and print all buffered records
    pub fn print_stored_records(&self) {
        let mut inner = self.inner.lock().unwrap();
        for stored_record in std::mem::take(&mut inner.stored_records) {
            println!(
                "[{} {} {}] {}",
                stored_record.time, stored_record.level, stored_record.mod_path, stored_record.text
            );
        }
    }

    /// Remove all stored logs
    pub fn flush_records(&self) {
        let mut inner = self.inner.lock().unwrap();
        inner.stored_records.clear();
    }
}

impl log::Log for Logger {
    fn enabled(&self, metadata: &log::Metadata<'_>) -> bool {
        let inner = self.inner.lock().unwrap();
        inner.env_logger.enabled(metadata)
    }

    fn log(&self, record: &log::Record<'_>) {
        if !self.enabled(record.metadata()) {
            return;
        }

        let mut inner = self.inner.lock().unwrap();

        if inner.buffer {
            let mod_path = record.module_path().unwrap_or("");
            inner.stored_records.push(StoredRecord {
                level: record.level(),
                time: chrono::Local::now(),
                mod_path: mod_path.to_owned(),
                text: record.args().to_string(),
            });
        } else {
            inner.env_logger.log(record);
        }
    }

    fn flush(&self) {}
}

/// Encapsulate caught unwound panics, such that we can catch tests that panic and differentiate
/// them from tests that just fail.
#[derive(Debug, thiserror::Error)]
#[error("Test panic: {}", self.as_string())]
pub struct Panic(Box<dyn std::any::Any + Send + 'static>);

impl Panic {
    /// Create a new [`Panic`] from a caught unwound panic.
    pub fn new(result: Box<dyn std::any::Any + Send + 'static>) -> Self {
        Self(result)
    }

    /// Convert this panic to a [`String`] representation.
    pub fn as_string(&self) -> String {
        if let Some(result) = self.0.downcast_ref::<String>() {
            return result.clone();
        }
        match self.0.downcast_ref::<&str>() {
            Some(s) => String::from(*s),
            None => String::from("unknown message"),
        }
    }
}

pub struct TestOutput {
    pub error_messages: Vec<Output>,
    pub test_name: &'static str,
    pub result: TestResult,
    pub log_output: Option<LogOutput>,
}

// Convert this unwieldy return type to a workable `TestResult`.
// What we are converting from is the acutal return type of the test execution.
impl From<Result<Result<(), Error>, Panic>> for TestResult {
    fn from(value: Result<Result<(), Error>, Panic>) -> Self {
        match value {
            Ok(Ok(())) => TestResult::Pass,
            Ok(Err(e)) => TestResult::Fail(e),
            Err(e) => TestResult::Panic(e),
        }
    }
}

/// Result from a test execution. This may carry information in case the test failed during
/// execution.
pub enum TestResult {
    /// Test passed.
    Pass,
    /// Test failed during execution. Contains the source error which caused the test to fail.
    Fail(Error),
    /// Test panicked during execution. Contains the caught unwound panic.
    Panic(Panic),
}

impl TestResult {
    /// Returns `true` if test failed or panicked, i.e. when `TestResult` is `Fail` or `Panic`.
    pub const fn failure(&self) -> bool {
        matches!(self, TestResult::Fail(_) | TestResult::Panic(_))
    }

    /// Convert `self` to a [`summary::TestResult`], which is used for creating fancy exports of
    /// the results for a test run.
    pub const fn summary(&self) -> summary::TestResult {
        match self {
            TestResult::Pass => summary::TestResult::Pass,
            TestResult::Fail(_) | TestResult::Panic(_) => summary::TestResult::Fail,
        }
    }

    /// Consume `self` and convert into a [`Result`] where [`TestResult::Pass`] is mapped to [`Ok`]
    /// while [`TestResult::Fail`] & [`TestResult::Panic`] is mapped to [`Err`].
    pub fn anyhow(self) -> anyhow::Result<()> {
        match self {
            TestResult::Pass => Ok(()),
            TestResult::Fail(error) => anyhow::bail!(error),
            TestResult::Panic(error) => anyhow::bail!(error.to_string()),
        }
    }
}

impl TestOutput {
    pub fn print(&self) {
        match &self.result {
            TestResult::Pass => {
                println!("{}", format!("TEST {} SUCCEEDED!", self.test_name).green());
                return;
            }
            TestResult::Fail(e) => {
                println!(
                    "{}",
                    format!(
                        "TEST {} RETURNED ERROR: {}",
                        self.test_name,
                        format!("{:?}", e).bold()
                    )
                    .red()
                );
            }
            TestResult::Panic(panic_msg) => {
                println!(
                    "{}",
                    format!(
                        "TEST {} PANICKED WITH MESSAGE: {}",
                        self.test_name,
                        panic_msg.as_string().bold()
                    )
                    .red()
                );
            }
        }

        println!("{}", format!("TEST {} HAD LOGS:", self.test_name).red());
        match &self.log_output {
            Some(log) => {
                match &log.settings_json {
                    Ok(settings) => println!("settings.json: {}", settings),
                    Err(e) => println!("Could not get settings.json: {}", e),
                }

                match &log.log_files {
                    Ok(log_files) => {
                        for log in log_files {
                            match log {
                                Ok(log) => {
                                    println!("Log {}:\n{}", log.name.to_str().unwrap(), log.content)
                                }
                                Err(e) => println!("Could not get log: {}", e),
                            }
                        }
                    }
                    Err(e) => println!("Could not get logs: {}", e),
                }
            }
            None => println!("Missing logs for {}", self.test_name),
        }

        println!(
            "{}",
            format!("TEST RUNNER {} HAD RUNTIME OUTPUT:", self.test_name).red()
        );
        if self.error_messages.is_empty() {
            println!("<no output>");
        } else {
            for msg in &self.error_messages {
                println!("{}", msg);
            }
        }

        println!("{}", format!("TEST {} END OF OUTPUT", self.test_name).red());
    }
}
