use std::{
    io::{self, Write},
    process::{Command, Stdio},
    result::Result::Ok,
};

pub struct FirewallConfiguration;

impl FirewallConfiguration {
    pub fn apply(
        &self,
        wan_if: &str,
        source_name: String,
        sink_name: String,
    ) -> Result<(), io::Error> {
        let anchor_name = "net.mullvad.raas";

        let anchors = include_str!("./pf_anchors.conf");
        let rules = include_str!("./pf_rules.conf");

        let script = include_str!("./apply.sh.template")
            .replace("{{ANCHORS}}", &anchors)
            .replace("{{RULES}}", &rules);

        let mut shell = Command::new("sh")
            .stdin(Stdio::piped())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .env("ANCHOR_NAME", anchor_name)
            .env("SINK_UTUN", &sink_name)
            .env("SOURCE_UTUN", &source_name)
            .env("WAN_IF", wan_if)
            .spawn()?;

        let mut stdin = shell
            .stdin
            .take()
            .ok_or_else(|| io::Error::other("failed to get stdin"))?;

        log::debug!("Source interface - {}", source_name);
        log::debug!("WAN interface - {}", wan_if);
        log::debug!("Sink interface - {}", sink_name);
        log::debug!(
            "Applying forwarding rules: {} -> {}, replies via {}",
            wan_if,
            source_name,
            sink_name
        );
        stdin.write_all(script.as_bytes())?;
        drop(stdin); // close the pipe so the shell sees EOF

        let status = shell.wait()?;

        if !status.success() {
            return Err(io::Error::other(format!(
                "Failed to apply firewall configuration: {status}"
            )));
        }

        Ok(())
    }

    pub fn reset(&self) -> Result<(), io::Error> {
        let mut shell = Command::new("sh")
            .stdin(Stdio::piped())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()?;

        let mut stdin = shell
            .stdin
            .take()
            .ok_or_else(|| io::Error::other("failed to get stdin"))?;

        let script = "pfctl -f '/etc/pf.conf'";

        log::info!("Loading main rules in /etc/pf.conf");
        stdin.write_all(script.as_bytes())?;
        drop(stdin); // close the pipe so the shell sees EOF
        //
        let status = shell.wait()?;
        if !status.success() {
            return Err(io::Error::other(format!(
                "Failed to load reset firewall to initial state. Something wrong in /etc/pf.conf? Failure: {status}"
            )));
        }

        Ok(())
    }
}
