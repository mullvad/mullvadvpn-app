use std::{collections::BTreeMap, io, path::Path};
use tokio::{
    fs,
    io::{AsyncBufReadExt, AsyncWriteExt},
};

#[derive(err_derive::Error, Debug)]
#[error(no_from)]
pub enum Error {
    #[error(display = "Failed to open log file {:?}", _1)]
    Open(#[error(source)] io::Error, std::path::PathBuf),
    #[error(display = "Failed to write to log file")]
    Write(#[error(source)] io::Error),
    #[error(display = "Failed to read from log file")]
    Read(#[error(source)] io::Error),
    #[error(display = "Failed to parse log file")]
    Parse,
}

#[derive(Clone, Copy)]
pub enum TestResult {
    Pass,
    Fail,
    Unknown,
}

impl TestResult {
    const PASS_STR: &'static str = "✅";
    const FAIL_STR: &'static str = "❌";
    const UNKNOWN_STR: &'static str = " ";
}

impl std::str::FromStr for TestResult {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            TestResult::PASS_STR => Ok(TestResult::Pass),
            TestResult::FAIL_STR => Ok(TestResult::Fail),
            _ => Ok(TestResult::Unknown),
        }
    }
}

impl std::fmt::Display for TestResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TestResult::Pass => f.write_str(TestResult::PASS_STR),
            TestResult::Fail => f.write_str(TestResult::FAIL_STR),
            TestResult::Unknown => f.write_str(TestResult::UNKNOWN_STR),
        }
    }
}

/// Logger that outputs test results in a structured format
pub struct SummaryLogger {
    file: fs::File,
}

impl SummaryLogger {
    /// Create a new logger and log to `path`. If `path` does not exist, it will be created. If it
    /// already exists, it is truncated and overwritten.
    pub async fn new(name: &str, path: &Path) -> Result<SummaryLogger, Error> {
        let mut file = fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(path)
            .await
            .map_err(|err| Error::Open(err, path.to_path_buf()))?;

        // The first row is the summary name
        file.write_all(name.as_bytes())
            .await
            .map_err(Error::Write)?;
        file.write_u8(b'\n').await.map_err(Error::Write)?;

        Ok(SummaryLogger { file })
    }

    pub async fn log_test_result(
        &mut self,
        test_name: &str,
        test_result: TestResult,
    ) -> Result<(), Error> {
        self.file
            .write_all(test_name.as_bytes())
            .await
            .map_err(Error::Write)?;
        self.file.write_u8(b' ').await.map_err(Error::Write)?;
        self.file
            .write_all(test_result.to_string().as_bytes())
            .await
            .map_err(Error::Write)?;
        self.file.write_u8(b'\n').await.map_err(Error::Write)?;

        Ok(())
    }
}

/// Convenience function that logs when there's a value, and is a no-op otherwise.
// y u no trait async fn
pub async fn maybe_log_test_result(
    summary_logger: Option<&mut SummaryLogger>,
    test_name: &str,
    test_result: TestResult,
) -> Result<(), Error> {
    match summary_logger {
        Some(logger) => logger.log_test_result(test_name, test_result).await,
        None => Ok(()),
    }
}

/// Parsed summary results
pub struct Summary {
    /// Summary name
    name: String,
    /// Pairs of test names mapped to test results
    results: BTreeMap<String, TestResult>,
}

impl Summary {
    /// Read test summary from `path`.
    pub async fn parse_log<P: AsRef<Path>>(path: P) -> Result<Summary, Error> {
        let file = fs::OpenOptions::new()
            .read(true)
            .open(&path)
            .await
            .map_err(|err| Error::Open(err, path.as_ref().to_path_buf()))?;

        let mut lines = tokio::io::BufReader::new(file).lines();

        let name = lines
            .next_line()
            .await
            .map_err(Error::Read)?
            .ok_or(Error::Parse)?;

        let mut results = BTreeMap::new();

        while let Some(line) = lines.next_line().await.map_err(Error::Read)? {
            let mut cols = line.split_whitespace();

            let test_name = cols.next().ok_or(Error::Parse)?;
            let test_result = cols.next().ok_or(Error::Parse)?.parse()?;

            results.insert(test_name.to_owned(), test_result);
        }

        Ok(Summary { name, results })
    }

    // Return all tests which passed.
    fn passed(&self) -> Vec<&TestResult> {
        self.results
            .values()
            .filter(|x| matches!(x, TestResult::Pass))
            .collect()
    }
}

/// Outputs an HTML table, to stdout, containing the results of the given log files.
///
/// This is a best effort attempt at summarizing the log files which do
/// exist. If some log file which is expected to exist, but for any reason fails to
/// be parsed, we should not abort the entire summarization.
pub async fn print_summary_table<P: AsRef<Path>>(summary_files: &[P]) {
    let mut summaries = Vec::new();
    let mut failed_to_parse = Vec::new();
    for sumfile in summary_files {
        match Summary::parse_log(sumfile).await {
            Ok(summary) => summaries.push(summary),
            Err(_) => failed_to_parse.push(sumfile),
        }
    }

    // Collect test details
    let tests: Vec<_> = inventory::iter::<crate::tests::TestMetadata>().collect();

    // Add some styling to the summary.
    println!("<head> <style> table, th, td {{ border: 1px solid black; }} </style> </head>");

    // Print a table
    println!("<table>");

    // First row: Print summary names
    println!("<tr>");
    println!("<td style='text-align: center;'>Test ⬇️ / Platform ➡️ </td>");

    for summary in &summaries {
        let total_tests = tests.len();
        let total_passed = summary.passed().len();
        let counter_text = if total_passed == total_tests {
            String::from(TestResult::PASS_STR)
        } else {
            format!("({}/{})", total_passed, total_tests)
        };
        println!(
            "<td style='text-align: center;'>{} {}</td>",
            summary.name, counter_text
        );
    }

    // A summary of all OSes
    println!("<td style='text-align: center;'>");
    println!("{}", {
        let oses_passed: Vec<_> = summaries
            .iter()
            .filter(|summary| summary.passed().len() == tests.len())
            .collect();
        if oses_passed.len() == summaries.len() {
            "🎉 All Platforms passed 🎉".to_string()
        } else {
            let failed: usize = summaries
                .iter()
                .map(|summary| {
                    if summary.passed().len() == tests.len() {
                        0
                    } else {
                        1
                    }
                })
                .sum();
            format!("🌧️ ️ {failed} Platform(s) failed 🌧️")
        }
    });
    println!("</td>");

    // List all tests again
    println!("<td style='text-align: center;'>Test ⬇️</td>");

    println!("</tr>");

    // Remaining rows: Print results for each test and each summary
    for test in &tests {
        println!("<tr>");

        println!(
            "<td>{}{}</td>",
            test.name,
            if test.must_succeed { " *" } else { "" }
        );

        let mut failed_platforms = vec![];
        for summary in &summaries {
            let result = summary
                .results
                .get(test.name)
                .unwrap_or(&TestResult::Unknown);
            match result {
                TestResult::Fail | TestResult::Unknown => {
                    failed_platforms.push(summary.name.clone())
                }
                TestResult::Pass => (),
            }
            println!("<td style='text-align: center;'>{}</td>", result);
        }
        // Print a summary of all OSes at the end of the table
        // For each test, collect the result for each platform.
        // - If the test passed on all platforms, we print a symbol declaring success
        // - If the test failed on any platform, we print the platform
        println!("<td style='text-align: center;'>");
        print!(
            "{}",
            if failed_platforms.is_empty() {
                TestResult::PASS_STR.to_string()
            } else {
                failed_platforms.join(", ")
            }
        );
        println!("</td>");

        // List the test name again (Useful for the summary accross the different platforms)
        println!("<td>{}</td>", test.name);

        // End row
        println!("</tr>");
    }

    println!("</table>");

    // Print explanation of test result
    println!("<p>{} = Test passed</p>", TestResult::PASS_STR);
    println!("<p>{} = Test failed</p>", TestResult::FAIL_STR);
}
