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

        // why u gotta be 'static
        let log_path2 = log_path.clone();
        tokio::task::spawn_blocking(move || rotate_log(&log_path2))
            .await
            .unwrap()
            .context("Failed to rotate log")?;

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

    fn parse_driverquery(out_s: String) -> anyhow::Result<String> {
        const KEEP_FIELDS: &[&str] = &[
            "\"Module Name\"",
            "\"Display Name\"",
            "\"Description\"",
            "\"Driver Type\"",
            "\"Start Mode\"",
            "\"Description\"",
            "\"State\"",
            "\"Status\"",
            "\"Path\"",
        ];
        parse_csv(&out_s, KEEP_FIELDS, keyword_filter_fn)
    }

    fn parse_pnputil(out_s: String) -> anyhow::Result<String> {
        const KEEP_FIELDS: &[&str] = &[
            "InstanceId",
            "DeviceDescription",
            "Status",
            "ProblemCode",
            "ProblemStatus",
            "DriverName",
        ];
        parse_csv(&out_s, KEEP_FIELDS, keyword_filter_fn)
    }

    fn parse_pnputil_problem(out_s: String) -> anyhow::Result<String> {
        const KEEP_FIELDS: &[&str] = &[
            "InstanceId",
            "DeviceDescription",
            "Status",
            "ProblemCode",
            "ProblemStatus",
            "DriverName",
        ];
        // In this case, we keep all entries
        parse_csv(&out_s, KEEP_FIELDS, |_| true)
    }

    fn keyword_filter_fn(line: &str) -> bool {
        KEYWORDS.iter().any(|word| {
            line.to_ascii_lowercase()
                .contains(&word.to_ascii_lowercase())
        })
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

    fn parse_csv(
        s: &str,
        keep_fields: &[&str],
        entry_filter_fn: impl Fn(&str) -> bool,
    ) -> anyhow::Result<String> {
        let mut lines = s.lines();
        // The first line contains the columns, so always keep it
        let first_line = lines.next().context("Empty output")?;

        let fields_to_keep: Vec<_> = first_line
            .split(',')
            .enumerate()
            .filter_map(|(ind, field)| {
                if keep_fields
                    .iter()
                    .any(|keep_f| field.eq_ignore_ascii_case(keep_f))
                {
                    return Some(ind);
                }
                None
            })
            .collect();

        let filtered_lines: Vec<_> = std::iter::once(first_line)
            // Filter out entries using entry_filter_fn
            .chain(lines.filter(|line| entry_filter_fn(line)))
            // Keep only fields of interest
            .map(|line| {
                line.split(',').enumerate().filter_map(|(ind, field)| {
                    if fields_to_keep.contains(&ind) {
                        return Some(field);
                    }
                    None
                }).collect::<Vec<_>>().join(",")
            })
            .collect();

        Ok(filtered_lines.join("\n"))
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
