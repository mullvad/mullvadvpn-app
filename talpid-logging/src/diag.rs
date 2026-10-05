//! Additional logging that may be useful

/// Additional logging for Windows
#[cfg(target_os = "windows")]
pub mod windows {
    use anyhow::Context;
    use anyhow::bail;
    use std::ffi::CStr;
    use std::ffi::OsString;
    use std::fmt::Write;
    use std::os::windows::ffi::OsStringExt;
    use std::path::Path;
    use talpid_windows::env::get_system_dir;
    use talpid_windows::string::multibyte_to_wide;
    use tokio::io::AsyncWriteExt;
    use tokio::process::Command;
    use windows_sys::Win32::Globalization::CP_ACP;

    /// Keywords used to filter output
    const KEYWORDS: &[&str] = &[
        "wireguard",
        "wintun",
        "tunnel",
        "mullvad",
        "split-tunnel",
        "split tunnel",
    ];

    /// Dump logs about tunnel devices and relevant drivers
    ///
    /// Currently, this will log the output of `pnputil` and `driverquery`, with filtering.
    pub async fn log_device_info(log_dir: &Path) -> anyhow::Result<()> {
        use crate::logging::rotate_log;
        use tokio::{fs::File, io::BufWriter};

        const TIMESTAMP_FMT: &str = "%Y-%m-%d %H:%M:%S";

        let log_path = log_dir.join("device.log");

        tokio::task::block_in_place(|| rotate_log(&log_path)).context("Failed to rotate log")?;

        let logger = File::options()
            .write(true)
            .create(true)
            .truncate(true)
            .open(log_path)
            .await
            .context("Failed to open device log")?;

        let mut logger = BufWriter::new(logger);

        // Log the current time
        logger
            .write_all(
                format!(
                    "Log time: {}\n\n",
                    chrono::Local::now().format(TIMESTAMP_FMT)
                )
                .as_bytes(),
            )
            .await?;

        async fn run_cmd_and_write_logs(
            logger: &mut BufWriter<File>,
            cmd: &mut Command,
            parse_output: impl FnOnce(String) -> anyhow::Result<String>,
        ) -> anyhow::Result<()> {
            let logs = run_cmd(cmd).await.and_then(parse_output);
            logger.write_all(format_logs(cmd, logs)?.as_bytes()).await?;
            Ok(())
        }

        run_cmd_and_write_logs(&mut logger, &mut driverquery_cmd()?, parse_driverquery).await?;
        run_cmd_and_write_logs(&mut logger, &mut pnputil_cmd()?, parse_pnputil).await?;
        run_cmd_and_write_logs(
            &mut logger,
            &mut pnputil_problem_cmd()?,
            parse_pnputil_problem,
        )
        .await?;

        let _ = logger.flush().await;

        Ok(())
    }

    /// Run `cmd` and collect its output
    async fn run_cmd(cmd: &mut Command) -> anyhow::Result<String> {
        let out = cmd.output().await.context("Failed to run driverquery")?;

        if !out.status.success() {
            bail!("driverquery failed: {:?}", out.status.code());
        }

        parse_raw_cmd_output(&out.stdout)
    }

    fn format_logs(cmd: &Command, output: anyhow::Result<String>) -> anyhow::Result<String> {
        let mut buf = String::new();

        writeln!(&mut buf, "{} (filtered)", format_command(cmd)?)?;
        writeln!(&mut buf, "--------")?;

        match output {
            Ok(out) => {
                writeln!(&mut buf, "{out}")?;
            }
            Err(err) => {
                writeln!(&mut buf, "The command failed due to an error: {err}")?;
            }
        }
        writeln!(&mut buf, "--------")?;

        Ok(buf)
    }

    /// Partial CSV records for `driverquery /FO csv ...`
    #[derive(serde::Deserialize, serde::Serialize)]
    struct DriverQueryRecords {
        #[serde(rename = "Module Name")]
        module_name: String,
        #[serde(rename = "Display Name")]
        display_name: String,
        #[serde(rename = "Description")]
        description: String,
        #[serde(rename = "Driver Type")]
        driver_type: String,
        #[serde(rename = "Start Mode")]
        start_mode: String,
        #[serde(rename = "State")]
        state: String,
        #[serde(rename = "Status")]
        status: String,
        #[serde(rename = "Path")]
        path: String,
    }

    fn parse_driverquery(out_s: String) -> anyhow::Result<String> {
        parse_csv::<DriverQueryRecords>(out_s.as_bytes(), driverquery_filter)
    }

    fn driverquery_filter(records: &DriverQueryRecords) -> bool {
        string_contains_keyword(&records.module_name)
            || string_contains_keyword(&records.display_name)
            || string_contains_keyword(&records.description)
            || string_contains_keyword(&records.path)
    }

    /// Partial CSV records for `pnputil /format csv ...`
    #[derive(serde::Deserialize, serde::Serialize)]
    #[serde(rename_all = "PascalCase")]
    struct PnputilRecords {
        instance_id: String,
        device_description: String,
        status: String,
        problem_code: String,
        problem_status: String,
        driver_name: String,
    }

    fn parse_pnputil(out_s: String) -> anyhow::Result<String> {
        parse_csv::<PnputilRecords>(out_s.as_bytes(), pnputil_filter)
    }

    fn parse_pnputil_problem(out_s: String) -> anyhow::Result<String> {
        // In this case, we keep all entries
        parse_csv::<PnputilRecords>(out_s.as_bytes(), |_| true)
    }

    fn pnputil_filter(records: &PnputilRecords) -> bool {
        string_contains_keyword(&records.instance_id)
            || string_contains_keyword(&records.device_description)
            || string_contains_keyword(&records.driver_name)
    }

    /// Return whether `s` contains one of the keywords in [KEYWORDS].
    /// This is case-insensitive.
    fn string_contains_keyword(s: &str) -> bool {
        KEYWORDS
            .iter()
            .any(|word| s.to_ascii_lowercase().contains(&word.to_ascii_lowercase()))
    }

    fn parse_raw_cmd_output(bytes: &[u8]) -> anyhow::Result<String> {
        // Convert from current codepage to UTF8
        // Seems not entirely correct, but probably good enough
        let mut bytes = bytes.to_vec();
        bytes.push(0);
        let bytes_cstr = CStr::from_bytes_until_nul(&bytes).unwrap();

        let str = multibyte_to_wide(bytes_cstr, CP_ACP).context("Invalid pnputil output")?;
        let out_s = OsString::from_wide(&str);
        Ok(out_s.to_string_lossy().into_owned())
    }

    fn parse_csv<RecordType: serde::de::DeserializeOwned + serde::Serialize>(
        data: &[u8],
        filter_fn: impl Fn(&RecordType) -> bool,
    ) -> anyhow::Result<String> {
        let mut csv = csv::Reader::from_reader(data);

        let mut buf = vec![];
        let mut out = csv::Writer::from_writer(&mut buf);

        csv.deserialize()
            .filter_map(|record_result| record_result.ok())
            .try_for_each(|record: RecordType| {
                if !filter_fn(&record) {
                    return Ok(());
                }
                out.serialize(record)
                    .context("Failed to serialize csv record")
            })?;

        drop(out);

        Ok(String::from_utf8_lossy(&buf).into_owned())
    }

    fn driverquery_cmd() -> anyhow::Result<Command> {
        let path = get_system_dir()?.join("driverquery.exe");
        let mut driver_query = Command::new(path);
        driver_query.args(["/FO", "csv", "/V"]);
        Ok(driver_query)
    }

    fn pnputil_cmd() -> anyhow::Result<Command> {
        let path = get_system_dir()?.join("pnputil.exe");
        let mut pnputil = Command::new(path);
        // Enumerate network devices
        pnputil.args([
            "/enum-devices",
            "/class",
            "{4d36e972-e325-11ce-bfc1-08002be10318}",
        ]);
        pnputil.args(["/format", "csv"]);
        Ok(pnputil)
    }

    fn pnputil_problem_cmd() -> anyhow::Result<Command> {
        let path = get_system_dir()?.join("pnputil.exe");
        let mut pnputil = Command::new(path);
        // Enumerate devices with issues
        pnputil.args(["/enum-devices", "/problem"]);
        pnputil.args(["/format", "csv"]);
        Ok(pnputil)
    }

    fn format_command(cmd: &Command) -> anyhow::Result<String> {
        let mut s = String::new();

        let prog = Path::new(cmd.as_std().get_program())
            .file_name()
            .context("Missing command filename")?
            .display();
        write!(&mut s, r#"{prog}"#)?;

        for arg in cmd.as_std().get_args() {
            write!(&mut s, r#" "{}""#, arg.display())?;
        }

        Ok(s)
    }

    #[cfg(test)]
    mod test {
        use super::*;

        /// Test whether driverquery output is filtered correctly
        #[tokio::test]
        async fn test_driverquery_output() {
            let test_output_path = Path::new(&std::env::var("CARGO_MANIFEST_DIR").unwrap())
                .join("src/logging/driverquery-out.testdata");

            // Uncomment to generate new output
            //tokio::fs::write(
            //    &test_output_path,
            //    driverquery_cmd().unwrap().output().await.unwrap().stdout,
            //)
            //.await
            //.unwrap();

            let my_output = std::fs::read(test_output_path).unwrap();
            let my_output = parse_raw_cmd_output(&my_output).unwrap();
            let parsed_driverquery_output = parse_driverquery(my_output).unwrap();
            let formatted_driverquery_log =
                format_logs(&driverquery_cmd().unwrap(), Ok(parsed_driverquery_output)).unwrap();

            insta::assert_snapshot!(formatted_driverquery_log);
        }

        /// Test whether pnputil output is filtered correctly
        #[tokio::test]
        async fn test_pnputil_output() {
            let test_output_path = Path::new(&std::env::var("CARGO_MANIFEST_DIR").unwrap())
                .join("src/logging/pnputil-out.testdata");

            // Uncomment to generate new output
            //tokio::fs::write(
            //    &test_output_path,
            //    pnputil_cmd().unwrap().output().await.unwrap().stdout,
            //)
            //.await
            //.unwrap();

            let my_output = std::fs::read(test_output_path).unwrap();
            let my_output = parse_raw_cmd_output(&my_output).unwrap();
            let parsed_output = parse_pnputil(my_output).unwrap();
            let formatted_logs = format_logs(&pnputil_cmd().unwrap(), Ok(parsed_output)).unwrap();

            insta::assert_snapshot!(formatted_logs);
        }

        /// Test whether pnputil output is filtered correctly
        #[tokio::test]
        async fn test_pnputil_problem_output() {
            let test_output_path = Path::new(&std::env::var("CARGO_MANIFEST_DIR").unwrap())
                .join("src/logging/pnputil-problem-out.testdata");

            // Uncomment to generate new output
            //tokio::fs::write(
            //    &test_output_path,
            //    pnputil_problem_cmd().unwrap().output().await.unwrap().stdout,
            //)
            //.await
            //.unwrap();

            let my_output = std::fs::read(test_output_path).unwrap();
            let my_output = parse_raw_cmd_output(&my_output).unwrap();
            let parsed_output = parse_pnputil_problem(my_output).unwrap();
            let formatted_logs =
                format_logs(&pnputil_problem_cmd().unwrap(), Ok(parsed_output)).unwrap();

            insta::assert_snapshot!(formatted_logs);
        }
    }
}
