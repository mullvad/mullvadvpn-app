#![cfg(target_os = "linux")]
use regex::Regex;
use std::{
    fs,
    io::{self, BufRead, BufReader, BufWriter, Read, Seek, Write},
    net::{AddrParseError, IpAddr},
    path::Path,
    process::Command,
    str::FromStr,
};
use talpid_types::SPLIT_TUNNEL_CGROUP_NAME;

const NETCLS_DIR: &str = "/sys/fs/cgroup/net_cls/";

/// Identifies packets coming from the cgroup.
/// This should be an arbitrary but unique integer.
pub const NETCLS_CLASSID: u32 = 0x4d9f41;
/// Value used to mark packets and associated connections.
/// This should be an arbitrary but unique integer.
pub const MARK: i32 = 0xf41;

const ROUTING_TABLE_NAME: &str = "mullvad_exclusions";
const RT_TABLES_PATH: &str = "/etc/iproute2/rt_tables";

/// Errors related to split tunneling.
#[derive(err_derive::Error, Debug)]
#[error(no_from)]
pub enum Error {
    /// Unable to list routing table entries.
    #[error(display = "Failed to enumerate routes")]
    EnumerateRoutes(#[error(source)] io::Error),

    /// Unable to find the interface/ip pair used by the physical interface.
    #[error(display = "No default route found")]
    NoDefaultRoute,

    /// Failed to parse string containing an IP address. May be invalid.
    #[error(display = "Failed to parse IP address")]
    ParseIpError(#[error(source)] AddrParseError),

    /// Failed to run the process.
    #[error(display = "Unable to execute process")]
    ExecFailed(#[error(source)] io::Error),

    /// ip command returned an error status.
    #[error(display = "ip command failed")]
    IpFailed,

    /// Unable to create routing table for tagged connections and packets.
    #[error(display = "Unable to create routing table")]
    RoutingTableSetup(#[error(source)] io::Error),

    /// Unable to create cgroup.
    #[error(display = "Unable to initialize net_cls cgroup instance")]
    InitNetClsCGroup(#[error(source)] nix::Error),

    /// Unable to create cgroup.
    #[error(display = "Unable to create cgroup for excluded processes")]
    CreateCGroup(#[error(source)] io::Error),

    /// Unable to set class ID for cgroup.
    #[error(display = "Unable to set cgroup class ID")]
    SetCGroupClassId(#[error(source)] io::Error),

    /// Unable to add PID to cgroup.procs.
    #[error(display = "Unable to add PID to cgroup.procs")]
    AddCGroupPid(#[error(source)] io::Error),

    /// Unable to remove PID to cgroup.procs.
    #[error(display = "Unable to remove PID from cgroup")]
    RemoveCGroupPid(#[error(source)] io::Error),

    /// Unable to read cgroup.procs.
    #[error(display = "Unable to obtain PIDs from cgroup.procs")]
    ListCGroupPids(#[error(source)] io::Error),

    /// Unable to add setup DNS routing.
    #[error(display = "Failed to add routing table DNS rules")]
    SetDns(#[error(source)] io::Error),
}

struct DefaultRoute {
    interface: String,
    address: IpAddr,
}

/// Obtain the IP/interface of the physical interface
fn get_default_route() -> Result<DefaultRoute, Error> {
    // FIXME: use netlink
    let mut cmd = Command::new("ip");
    cmd.args(&["-4", "route", "list", "table", "main"]);
    log::trace!("running cmd - {:?}", &cmd);
    let out = cmd.output().map_err(Error::EnumerateRoutes)?;
    let out_str = String::from_utf8_lossy(&out.stdout);

    // Find "default" row
    let expression = Regex::new(r"^default via ([0-9.]+) dev (\w+)").unwrap();

    for line in out_str.lines() {
        if let Some(captures) = expression.captures(&line) {
            let ip_str = captures.get(1).unwrap().as_str();
            let interface = captures.get(2).unwrap().as_str().to_string();

            return Ok(DefaultRoute {
                interface,
                address: IpAddr::from_str(ip_str).map_err(Error::ParseIpError)?,
            });
        }
    }

    Err(Error::NoDefaultRoute)
}

/// Manage routing for split tunneling cgroup.
pub struct SplitTunnel {
    table_id: i32,
}

impl SplitTunnel {
    /// Object that allows specified applications to not pass through the tunnel
    pub fn new() -> Result<SplitTunnel, Error> {
        let mut tunnel = SplitTunnel { table_id: 0 };
        tunnel.initialize_routing_table()?;
        Ok(tunnel)
    }

    /// Set up policy-based routing for marked packets.
    fn initialize_routing_table(&mut self) -> Result<(), Error> {
        // Add routing table to /etc/iproute2/rt_tables, if it does not exist

        let file = fs::OpenOptions::new()
            .read(true)
            .open(RT_TABLES_PATH)
            .map_err(Error::RoutingTableSetup)?;
        let buf_reader = BufReader::new(file);
        let expression = Regex::new(r"^\s*(\d+)\s+(\w+)").unwrap();

        let mut used_ids = Vec::<i32>::new();

        for line in buf_reader.lines() {
            let line = line.map_err(Error::RoutingTableSetup)?;
            if let Some(captures) = expression.captures(&line) {
                let table_id = captures
                    .get(1)
                    .unwrap()
                    .as_str()
                    .parse::<i32>()
                    .expect("Table ID does not fit i32");
                let table_name = captures.get(2).unwrap().as_str();

                if table_name == ROUTING_TABLE_NAME {
                    // The table has already been added
                    self.table_id = table_id;
                    return Ok(());
                }

                used_ids.push(table_id);
            }
        }

        used_ids.sort_unstable();
        for id in 1..256 {
            if used_ids.binary_search(&id).is_err() {
                // Assign a free id to the table
                self.table_id = id;
                break;
            }
        }

        let mut file = fs::OpenOptions::new()
            .read(true)
            .append(true)
            .open(RT_TABLES_PATH)
            .map_err(Error::RoutingTableSetup)?;

        if let Ok(_) = file.seek(io::SeekFrom::End(-1)) {
            // Append newline if necessary
            let mut buffer = [0u8];
            let _ = file.read_exact(&mut buffer);
            if buffer[0] != b'\n' {
                writeln!(file).map_err(Error::RoutingTableSetup)?;
            }
        }

        writeln!(file, "{} {}", self.table_id, ROUTING_TABLE_NAME).map_err(Error::RoutingTableSetup)
    }

    /// Reset the split-tunneling routing table to its default state
    fn reset_table() -> Result<(), Error> {
        let _ = exec_ip(&["-4", "route", "flush", "table", ROUTING_TABLE_NAME]);

        // Force routing through the physical interface
        let default_route = get_default_route()?;
        exec_ip(&[
            "-4",
            "route",
            "add",
            "default",
            "via",
            &default_route.address.to_string(),
            "dev",
            &default_route.interface,
            "table",
            ROUTING_TABLE_NAME,
        ])
    }

    /// Route PID-associated packets through the physical interface.
    pub fn enable_routing(&self) -> Result<(), Error> {
        // TODO: IPv6

        // Create the rule if it does not exist
        let mut cmd = Command::new("ip");
        cmd.args(&["-4", "rule", "list", "table", ROUTING_TABLE_NAME]);
        log::trace!("running cmd - {:?}", &cmd);
        let out = cmd.output().map_err(Error::ExecFailed)?;

        let missing_rule =
            !out.status.success() || String::from_utf8_lossy(&out.stdout).trim().is_empty();
        if missing_rule {
            exec_ip(&[
                "-4",
                "rule",
                "add",
                "from",
                "all",
                "fwmark",
                &MARK.to_string(),
                "lookup",
                ROUTING_TABLE_NAME,
            ])?;
        }

        Self::reset_table()
    }

    /// Stop routing PID-associated packets through the physical interface.
    pub fn disable_routing(&self) -> Result<(), Error> {
        // TODO: IPv6

        if let Err(e) = exec_ip(&[
            "-4",
            "rule",
            "del",
            "from",
            "all",
            "fwmark",
            &MARK.to_string(),
            "lookup",
            ROUTING_TABLE_NAME,
        ]) {
            log::warn!("Failed to delete routing policy: {}", e);
        }

        Ok(())
    }

    /// Route DNS requests through the tunnel interface.
    pub fn route_dns(&self, tunnel_alias: &str, dns_servers: &[IpAddr]) -> Result<(), Error> {
        Self::reset_table()?;

        for server in dns_servers {
            if let IpAddr::V4(addr) = server {
                let addr = addr.to_string();
                exec_ip(&[
                    "-4",
                    "route",
                    "add",
                    &addr,
                    "via",
                    &addr,
                    "dev",
                    tunnel_alias,
                    "table",
                    ROUTING_TABLE_NAME,
                ])?;
            }
        }

        Ok(())
    }

    /// Reset DNS rules.
    pub fn flush_dns(&self) -> Result<(), Error> {
        // For now, simply flush it
        Self::reset_table()
    }
}

/// Manages PIDs to exclude from the tunnel.
pub struct PidManager;

impl PidManager {
    /// Create object to manage split-tunnel PIDs.
    pub fn new() -> Result<PidManager, Error> {
        Self::create_cgroup()?;
        Ok(PidManager {})
    }

    /// Set up cgroup used to track PIDs for split tunneling.
    fn create_cgroup() -> Result<(), Error> {
        let netcls_dir = Path::new(NETCLS_DIR);
        if !netcls_dir.exists() {
            fs::create_dir(netcls_dir.clone()).map_err(Error::CreateCGroup)?;

            // https://www.kernel.org/doc/Documentation/cgroup-v1/net_cls.txt
            nix::mount::mount(
                Some("net_cls"),
                netcls_dir,
                Some("cgroup"),
                nix::mount::MsFlags::empty(),
                Some("net_cls"),
            )
            .map_err(Error::InitNetClsCGroup)?;
        }

        let exclusions_dir = netcls_dir.join(SPLIT_TUNNEL_CGROUP_NAME);

        if !exclusions_dir.exists() {
            fs::create_dir(exclusions_dir.clone()).map_err(Error::CreateCGroup)?;
        }

        let classid_path = exclusions_dir.join("net_cls.classid");
        fs::write(classid_path, NETCLS_CLASSID.to_string().as_bytes())
            .map_err(Error::SetCGroupClassId)
    }

    /// Add a PID to exclude from the tunnel.
    pub fn add(&self, pid: i32) -> Result<(), Error> {
        self.add_list(&[pid])
    }

    /// Add PIDs to exclude from the tunnel.
    pub fn add_list(&self, pids: &[i32]) -> Result<(), Error> {
        let exclusions_path = Path::new(NETCLS_DIR)
            .join(SPLIT_TUNNEL_CGROUP_NAME)
            .join("cgroup.procs");

        let file = fs::OpenOptions::new()
            .write(true)
            .create(true)
            .open(exclusions_path)
            .map_err(Error::AddCGroupPid)?;

        let mut writer = BufWriter::new(file);

        for pid in pids {
            writer
                .write_all(pid.to_string().as_bytes())
                .map_err(Error::AddCGroupPid)?;
        }

        Ok(())
    }

    /// Remove a PID from processes to exclude from the tunnel.
    pub fn remove(&self, pid: i32) -> Result<(), Error> {
        // FIXME: We remove PIDs from our cgroup here by adding
        //        them to the parent cgroup. This seems wrong.
        let exclusions_path = Path::new(NETCLS_DIR)
            .join(SPLIT_TUNNEL_CGROUP_NAME)
            .join("cgroup.procs");

        let mut file = fs::OpenOptions::new()
            .write(true)
            .create(true)
            .open(exclusions_path)
            .map_err(Error::RemoveCGroupPid)?;

        file.write_all(pid.to_string().as_bytes())
            .map_err(Error::RemoveCGroupPid)
    }

    /// Return a list of PIDs that are excluded from the tunnel.
    pub fn list(&self) -> Result<Vec<i32>, Error> {
        // TODO: manage child PIDs somehow?

        let exclusions_path = Path::new(NETCLS_DIR)
            .join(SPLIT_TUNNEL_CGROUP_NAME)
            .join("cgroup.procs");

        let file = fs::File::open(exclusions_path).map_err(Error::ListCGroupPids)?;

        let result: Result<Vec<i32>, io::Error> = BufReader::new(file)
            .lines()
            .map(|line| {
                line.and_then(|v| {
                    v.parse()
                        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
                })
            })
            .collect();
        result.map_err(Error::ListCGroupPids)
    }

    /// Clear list of PIDs to exclude from the tunnel.
    pub fn clear(&self) -> Result<(), Error> {
        // TODO: reuse file handle
        let pids = self.list()?;

        for pid in pids {
            self.remove(pid)?;
        }

        Ok(())
    }
}

fn exec_ip(args: &[&str]) -> Result<(), Error> {
    let mut cmd = Command::new("ip");
    cmd.args(args);

    log::trace!("running cmd - {:?}", &cmd);

    let status = cmd.status().map_err(Error::ExecFailed)?;
    if status.success() {
        Ok(())
    } else {
        Err(Error::IpFailed)
    }
}
