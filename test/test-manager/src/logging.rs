use crate::tests::Error;
use colored::Colorize;
use std::sync::{Arc, Mutex};
use test_rpc::logging::{LogOutput, Output};

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
            logger.filter_level(log::LevelFilter::Debug);
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

#[derive(Debug, err_derive::Error)]
#[error(display = "Test panic: {}", _0)]
pub struct PanicMessage(String);

pub struct TestOutput {
    pub error_messages: Vec<Output>,
    pub test_name: &'static str,
    pub result: Result<Result<(), Error>, PanicMessage>,
    pub log_output: Option<LogOutput>,
}

impl TestOutput {
    pub fn print(&self) {
        match &self.result {
            Ok(Ok(_)) => {
                println!("{}", format!("TEST {} SUCCEEDED!", self.test_name).green());
                return;
            }
            Ok(Err(e)) => {
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
            Err(panic_msg) => {
                println!(
                    "{}",
                    format!(
                        "TEST {} PANICKED WITH MESSAGE: {}",
                        self.test_name,
                        panic_msg.0.bold()
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

pub fn panic_as_string(error: Box<dyn std::any::Any + Send + 'static>) -> PanicMessage {
    if let Some(result) = error.downcast_ref::<String>() {
        return PanicMessage(result.clone());
    }
    match error.downcast_ref::<&str>() {
        Some(s) => PanicMessage(String::from(*s)),
        None => PanicMessage(String::from("unknown message")),
    }
}
