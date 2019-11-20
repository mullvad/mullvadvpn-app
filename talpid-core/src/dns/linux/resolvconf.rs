use std::{
    collections::{BTreeMap, BTreeSet, HashSet},
    ffi::OsStr,
    fs, io,
    net::IpAddr,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    str::FromStr,
};

use super::RESOLV_CONF_PATH;
use regex::Regex;
use which::which;

pub type Result<T> = std::result::Result<T, Error>;

lazy_static::lazy_static! {
    static ref RESOLVCONF_NAMESERVER_REGEX: Regex =
        Regex::new(r"nameserver (.*)").expect("Failed to initialize resolvconf regex");
    static ref SYSTEMD_RESOLVED_STATUS_LINK_LINE: Regex =
        Regex::new(r"Link \d* \((.*)\)").expect("Failed to initialize resolvconf regex");
    static ref SYSTEMD_RESOLVED_STATUS_DNS_SERVER_LINE: Regex =
        Regex::new(r". DNS Servers: (.*)").expect("Failed to initialize resolvconf regex");
}

#[derive(err_derive::Error, Debug)]
pub enum Error {
    #[error(display = "Failed to detect 'resolvconf' program")]
    NoResolvconf,

    #[error(display = "The resolvconf in PATH is just a symlink to systemd-resolved")]
    ResolvconfUsesResolved,

    #[error(display = "Failed to execute 'resolvconf' program")]
    RunResolvconf(#[error(source)] io::Error),

    #[error(display = "Using 'resolvconf' to add a record failed: {}", stderr)]
    AddRecordError { stderr: String },

    #[error(display = "Using 'resolvconf' to delete a record failed")]
    DeleteRecordError,

    #[error(display = "Failed to verify if 'resolvconf' actually set ")]
    VerificationError,
}

pub struct Resolvconf {
    record_names: HashSet<String>,
    resolvconf: PathBuf,
}

impl Resolvconf {
    pub fn new() -> Result<Self> {
        let resolvconf_path = which("resolvconf").map_err(|_| Error::NoResolvconf)?;
        if Self::resolvconf_is_resolved_symlink(&resolvconf_path) {
            return Err(Error::ResolvconfUsesResolved);
        }
        Ok(Resolvconf {
            record_names: HashSet::new(),
            resolvconf: resolvconf_path,
        })
    }

    fn resolvconf_is_resolved_symlink(resolvconf_path: &Path) -> bool {
        fs::read_link(resolvconf_path)
            .map(|resolvconf_target| {
                resolvconf_target.file_name() == Some(OsStr::new("resolvectl"))
            })
            .unwrap_or_else(|_| false)
    }

    pub fn set_dns(&mut self, interface: &str, servers: &[IpAddr]) -> Result<()> {
        let record_name = format!("{}.mullvad", interface);
        let mut record_contents = String::new();

        for address in servers {
            record_contents.push_str("nameserver ");
            record_contents.push_str(&address.to_string());
            record_contents.push('\n');
        }

        let output = duct::cmd!(&self.resolvconf, "-a", &record_name)
            .stdin_bytes(record_contents)
            .stderr_capture()
            .unchecked()
            .run()
            .map_err(Error::RunResolvconf)?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            return Err(Error::AddRecordError { stderr });
        }

        self.record_names.insert(record_name);

        if !(Self::verify_systemd_resolved_output(interface, servers)
            || Self::verify_resolvconf(servers)
            || self.verify_output_of_resolvconf(servers))
        {
            log::error!("Couldn't verify DNS servers with systemd-resolve");
            return Err(Error::VerificationError);
        }

        Ok(())
    }

    pub fn reset(&mut self) -> Result<()> {
        let mut result = Ok(());

        for record_name in self.record_names.drain() {
            let output = duct::cmd!(&self.resolvconf, "-d", &record_name)
                .stderr_capture()
                .unchecked()
                .run()
                .map_err(Error::RunResolvconf)?;

            if !output.status.success() {
                log::error!(
                    "Failed to delete 'resolvconf' record '{}':\n{}",
                    record_name,
                    String::from_utf8_lossy(&output.stderr)
                );
                result = Err(Error::DeleteRecordError);
            }
        }

        result
    }

    // Whilst systemd-resolved has a DBus interface, it is not being used here since
    // if the DBus interface was usable, Resolvconf wouldn't be used. As such,
    // this function tries to determine whether the invocation of resolvconf resulted
    // in systemd-resolved applying our DNS config by running `systemd-resolved --status`
    // and parsing it's output
    fn verify_systemd_resolved_output(interface: &str, servers: &[IpAddr]) -> bool {
        let systemd_resolved_path = match which::which("systemd-resolved") {
            Ok(path) => path,
            Err(e) => {
                log::trace!("Failed to get path for systemd-resolved binary: {}", e);
                return false;
            }
        };
        let resolved_status = match Command::new(systemd_resolved_path)
            .arg("--status")
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .spawn()
        {
            Ok(child_process) => child_process,
            Err(e) => {
                log::error!("Failed to spawn systemd-resolevd child process: {}", e);
                return false;
            }
        };
        match resolved_status.wait_with_output() {
            Ok(output) => {
                let output = String::from_utf8_lossy(&output.stdout);
                let link_map = Self::parse_systemd_resolved_status_output(&output);
                link_map
                    .get(interface)
                    .map(|set_dns_servers| {
                        servers
                            .iter()
                            .all(|server| set_dns_servers.contains(server))
                    })
                    .unwrap_or(false)
            }

            Err(e) => {
                log::error!("Executing systemd-resolved failed - {}", e);
                false
            }
        }
    }

    // The output of the systemd-resolved --status command:
    //>Global
    //>   DNSSEC NTA: 10.in-addr.arpa
    //>               16.172.in-addr.arpa
    //> Link 2 (enp1s0)
    //>      Current Scopes: DNS
    //>       LLMNR setting: yes
    //> MulticastDNS setting: no
    //>      DNSSEC setting: no
    //>    DNSSEC supported: no
    //>         DNS Servers: 192.168.122.1
    //>                      192.168.1.1
    //>                      8.8.8.8
    //>          DNS Domain: ~.
    //
    fn parse_systemd_resolved_status_output<'a>(
        output: &'a str,
    ) -> BTreeMap<&'a str, BTreeSet<IpAddr>> {
        let mut link = None;
        let mut link_map = BTreeMap::new();
        let mut line_iter = output.lines().peekable();
        while let Some(line) = line_iter.next() {
            // extract current link from line like 'Link 2 (enp1s0)'
            let new_link = Self::get_first_match(line, &SYSTEMD_RESOLVED_STATUS_LINK_LINE);
            if new_link.is_some() {
                link = new_link;
            }

            // extract first DNS server from line like '         DNS Servers: 192.168.122.1'
            if link.is_some() && line.trim_start().starts_with("DNS Servers") {
                let link = link.unwrap();

                match Self::get_first_match(line, &SYSTEMD_RESOLVED_STATUS_DNS_SERVER_LINE) {
                    Some(ip_str) => match IpAddr::from_str(ip_str) {
                        Ok(ip_addr) => {
                            let entry = link_map.entry(link).or_insert(BTreeSet::new());
                            entry.insert(ip_addr);
                        }
                        Err(err) => {
                            log::error!("Failed to parse IP address from line '{}', {}", line, err);
                            continue;
                        }
                    },
                    None => continue,
                };
                'inner: while let Some(line) = line_iter.peek() {
                    match IpAddr::from_str(line.trim()) {
                        Ok(ip_addr) => {
                            link_map
                                .entry(link)
                                .or_insert(BTreeSet::new())
                                .insert(ip_addr);
                            // on success, move the iterator forward
                            line_iter.next();
                        }
                        // when the next line no longer has IP addresses,
                        Err(_) => {
                            break 'inner;
                        }
                    }
                }
            }
        }

        link_map
    }

    fn get_first_match<'a>(line: &'a str, re: &Regex) -> Option<&'a str> {
        re.captures(line)
            .and_then(|cap| cap.get(1))
            .map(|cap_match| cap_match.as_str())
    }

    fn verify_output_of_resolvconf(&self, expected_dns_servers: &[IpAddr]) -> bool {
        let resolvconf_proc = match Command::new(&self.resolvconf)
            .arg("-l")
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .spawn()
        {
            Ok(child_process) => child_process,
            Err(e) => {
                log::error!("Failed to spawn systemd-resolevd child process: {}", e);
                return false;
            }
        };

        match resolvconf_proc.wait_with_output() {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let set_dns_servers = Self::parse_resolvconf(&stdout);
                expected_dns_servers
                    .iter()
                    .all(|server| set_dns_servers.contains(server))
            }
            Err(e) => {
                log::error!("'resolvconf -l' return a non-zero exit code: {}", e);
                false
            }
        }
    }

    fn verify_resolvconf(expected_dns_servers: &[IpAddr]) -> bool {
        let buf = match fs::read(RESOLV_CONF_PATH) {
            Ok(f) => f,
            Err(e) => {
                log::error!("Failed to read {} - {}", RESOLV_CONF_PATH, e);
                return false;
            }
        };
        let resolvconf_contents = String::from_utf8_lossy(&buf);
        let set_dns_servers = Self::parse_resolvconf(&resolvconf_contents);
        expected_dns_servers
            .iter()
            .all(|server| set_dns_servers.contains(server))
    }

    fn parse_resolvconf(output: &str) -> BTreeSet<IpAddr> {
        output
            .lines()
            .filter_map(|line| Self::get_first_match(line, &RESOLVCONF_NAMESERVER_REGEX))
            .filter_map(|ip_str| IpAddr::from_str(ip_str).ok())
            .collect()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::iter::FromIterator;

    #[test]
    fn test_systemd_resolved_output_parsing() {
        let output_str = r#"Global
          DNSSEC NTA: 10.in-addr.arpa
                      16.172.in-addr.arpa
                      168.192.in-addr.arpa
                      17.172.in-addr.arpa
                      18.172.in-addr.arpa
                      19.172.in-addr.arpa
                      20.172.in-addr.arpa
                      21.172.in-addr.arpa
                      22.172.in-addr.arpa
                      23.172.in-addr.arpa
                      24.172.in-addr.arpa
                      25.172.in-addr.arpa
                      26.172.in-addr.arpa
                      27.172.in-addr.arpa
                      28.172.in-addr.arpa
                      29.172.in-addr.arpa
                      30.172.in-addr.arpa
                      31.172.in-addr.arpa
                      corp
                      d.f.ip6.arpa
                      home
                      internal
                      intranet
                      lan
                      local
                      private
                      test

Link 9170 (tun0)
      Current Scopes: DNS
       LLMNR setting: yes
MulticastDNS setting: no
      DNSSEC setting: no
    DNSSEC supported: no
         DNS Servers: 10.64.0.1
                      10.64.0.2
          DNS Domain: ~.

Link 2 (enp1s0)
      Current Scopes: DNS
       LLMNR setting: yes
MulticastDNS setting: no
      DNSSEC setting: no
    DNSSEC supported: no
         DNS Servers: 192.168.122.1
                      192.168.1.1
                      8.8.8.8
"#;
        let expected_map = BTreeMap::from_iter(
            vec![
                (
                    "enp1s0",
                    BTreeSet::from_iter(
                        vec![
                            "192.168.122.1".parse().unwrap(),
                            "192.168.1.1".parse().unwrap(),
                            "8.8.8.8".parse().unwrap(),
                        ]
                        .into_iter(),
                    ),
                ),
                (
                    "tun0",
                    BTreeSet::from_iter(
                        vec!["10.64.0.1".parse().unwrap(), "10.64.0.2".parse().unwrap()]
                            .into_iter(),
                    ),
                ),
            ]
            .into_iter(),
        );

        let map = Resolvconf::parse_systemd_resolved_status_output(output_str);
        assert_eq!(expected_map, map);
    }

    #[test]
    fn test_systemd_resolved_output_parsing_empty_output() {
        let output_str = r#"Global
          DNSSEC NTA: 10.in-addr.arpa
                      16.172.in-addr.arpa
                      168.192.in-addr.arpa
                      17.172.in-addr.arpa

"#;
        let expected_map = BTreeMap::new();
        let map = Resolvconf::parse_systemd_resolved_status_output(output_str);
        assert_eq!(expected_map, map);
    }

    #[test]
    fn test_systemd_resolved_output_parsing_invalid_output() {
        let output_str = r#"Global
          DNSSEC NTA: 10.in-addr.arpa
                      16.172.in-addr.arpa
                      168.192.in-addr.arpa
                      17.172.in-addr.arpa

         DNS Servers: 10.64.0.1
                      10.64.0.2
"#;
        let expected_map = BTreeMap::new();
        let map = Resolvconf::parse_systemd_resolved_status_output(output_str);
        assert_eq!(expected_map, map);
    }
}
