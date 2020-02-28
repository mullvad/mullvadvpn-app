use regex::Regex;
use std::{
    fs,
    io::{self, BufRead, BufReader, Write},
    path::Path,
    process::Command,
};

const NETCLS_DIR: &str = "/sys/fs/cgroup/net_cls/";

/// Identifies packets coming from the cgroup.
pub const NETCLS_CLASSID: u32 = 0x4d9f41;
/// Value used to mark packets and associated connections.
pub const MARK: i32 = 0xf41;

const CGROUP_NAME: &str = "mullvad-exclusions";
static mut ROUTING_TABLE_ID: i32 = 19;
const ROUTING_TABLE_NAME: &str = "mullvad_exclusions";
const RT_TABLES_PATH: &str = "/etc/iproute2/rt_tables";

/// Errors related to split tunneling.
#[derive(err_derive::Error, Debug)]
#[error(no_from)]
pub enum Error {
    /// Unable to create routing table for tagged connections and packets.
    #[error(display = "Unable to create routing table")]
    RoutingTableSetup(#[error(source)] io::Error),

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
}

fn route_marked_packets() -> Result<(), Error> {
    // TODO: IPv6
    let mut cmd = Command::new("ip");
    cmd.args(&[
        "-4",
        "rule",
        "add",
        "from",
        "all",
        "fwmark",
        &MARK.to_string(),
        "lookup",
        ROUTING_TABLE_NAME,
    ]);

    log::trace!("running cmd - {:?}", &cmd);
    cmd.output().map(|_| ()).map_err(Error::RoutingTableSetup)
}

/// Set up policy-based routing for marked packets.
pub fn initialize_routing_table() -> Result<(), Error> {
    // TODO: ensure the ID does not conflict with that of another table

    // Add routing table to /etc/iproute2/rt_tables, if it does not exist

    let mut file = fs::OpenOptions::new()
        .read(true)
        .append(true)
        .create(true)
        .open(RT_TABLES_PATH)
        .map_err(Error::RoutingTableSetup)?;
    let buf_reader = BufReader::new(file.try_clone().map_err(Error::RoutingTableSetup)?);
    let expression = Regex::new(r"^\s*(\d+)\s+(\w+)").unwrap();

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

            // Already added
            if table_name == ROUTING_TABLE_NAME {
                if table_id != unsafe { ROUTING_TABLE_ID } {
                    unsafe { ROUTING_TABLE_ID = table_id };
                }

                return route_marked_packets();
            }
        }
    }

    write!(
        file,
        "{} {}",
        unsafe { ROUTING_TABLE_ID },
        ROUTING_TABLE_NAME
    )
    .map_err(Error::RoutingTableSetup)?;

    route_marked_packets()
}

/// Set up cgroup used to track PIDs for split tunneling.
pub fn create_cgroup() -> Result<(), Error> {
    let exclusions_dir = Path::new(NETCLS_DIR).join(CGROUP_NAME);

    if !exclusions_dir.exists() {
        fs::create_dir(exclusions_dir.clone()).map_err(Error::CreateCGroup)?;
    }

    let classid_path = exclusions_dir.join("net_cls.classid");
    fs::write(classid_path, NETCLS_CLASSID.to_string().as_bytes()).map_err(Error::SetCGroupClassId)
}

/// Add a PID to exclude from the tunnel.
pub fn add_pid(pid: i32) -> Result<(), Error> {
    let exclusions_path = Path::new(NETCLS_DIR).join(CGROUP_NAME).join("cgroup.procs");

    let mut file = fs::OpenOptions::new()
        .write(true)
        .create(true)
        .open(exclusions_path)
        .map_err(Error::AddCGroupPid)?;

    file.write_all(pid.to_string().as_bytes())
        .map_err(Error::AddCGroupPid)
}

/// Remove a PID from processes to exclude from the tunnel.
pub fn remove_pid(pid: i32) -> Result<(), Error> {
    // FIXME: We remove PIDs from our cgroup here by adding
    //        them to the parent cgroup. This seems wrong.
    let exclusions_path = Path::new(NETCLS_DIR).join("cgroup.procs");

    let mut file = fs::OpenOptions::new()
        .write(true)
        .create(true)
        .open(exclusions_path)
        .map_err(Error::RemoveCGroupPid)?;

    file.write_all(pid.to_string().as_bytes())
        .map_err(Error::RemoveCGroupPid)
}

/// Return a list of PIDs that are excluded from the tunnel.
pub fn list_pids() -> Result<Vec<i32>, Error> {
    let exclusions_path = Path::new(NETCLS_DIR).join(CGROUP_NAME).join("cgroup.procs");

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
pub fn clear_pids() -> Result<(), Error> {
    // TODO: reuse file handle
    let pids = list_pids()?;

    for pid in pids {
        remove_pid(pid)?;
    }

    Ok(())
}
