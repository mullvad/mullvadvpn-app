fn main() {
    #[cfg(target_os = "linux")]
    inner::main();
}

#[cfg(target_os = "linux")]
mod inner {
    use anyhow::{Context, anyhow, bail};
    use libbpf_rs::{
        ObjectBuilder, Program,
        libbpf_sys::{bpf_attach_type, bpf_prog_attach},
    };
    use nix::unistd::{execvp, getgid, getuid, setegid, seteuid, setgid, setuid};
    use std::{
        env::args_os,
        ffi::{CString, OsString},
        fs::remove_file,
        io,
        os::{fd::AsRawFd, unix::ffi::OsStringExt as _},
        path::Path,
    };
    use talpid_cgroup::v2::CGroup2;

    mod bpf_programs {
        // TODO: move to dist-assets/binaries
        pub static EXCLUDE_CGROUP_SOCK: &[u8] = include_bytes!(concat!(
            "../bpf/mullvad-exclude.cgroup-sock-create.bpf.",
            env!("CARGO_CFG_TARGET_ARCH"),
        ));
    }

    // TODO: comment
    pub fn main() {
        if let Err(e) = run() {
            eprintln!("{e:?}");
            std::process::exit(1);
        }
    }

    /// Get the [`CGroup2`] of the current process.
    fn get_current_cgroup() -> anyhow::Result<CGroup2> {
        let cgroup_file = std::fs::read_to_string("/proc/self/cgroup")
            .context("Failed to read /proc/self/cgroup")?;

        /// Parse a line from /proc/<pid>/cgroup. See `man cgroup(7)`
        fn parse_line(line: &str) -> Option<(&str, &str, &str)> {
            let line = line.trim();
            let (hierarchy_id, line) = line.split_once(':')?;
            let (controller_list, cgroup_path) = line.split_once(':')?;
            Some((hierarchy_id, controller_list, cgroup_path))
        }

        let (_, _, cgroup_path) = cgroup_file
            .lines()
            .filter_map(parse_line)
            .filter(|&(hierarchy_id, _, _)| hierarchy_id == "0") // cgroup2 hierarchy_id is 0
            .next()
            .context("Expected a line starting with '0::/' containing the cgroup2 path")
            .context("Failed to parse /proc/self/cgroup")?;
        let cgroup_fs_path = Path::new("/sys/fs/cgroup").join(cgroup_path.trim_start_matches('/'));
        let cgroup = CGroup2::open(cgroup_fs_path).context("Failed to open cgroup2")?;

        Ok(cgroup)
    }

    /// Load [`bpf_programs::EXCLUDE_CGROUP_SOCK`] into the kernel and attach it to `cgroup`.
    ///
    /// The program will stay loaded and attached until the cgroup is destroyed when the last
    /// grouped process exits.
    fn install_exclusion_bpf_for_cgroup(cgroup: &CGroup2) -> anyhow::Result<()> {
        // Load the eBPF ELF-file into the kernel.
        let program = ObjectBuilder::default()
            .debug(false)
            .open_memory(bpf_programs::EXCLUDE_CGROUP_SOCK)?
            .load()
            .context("Failed to load eBPF program")?;

        for mut program in program.progs_mut() {
            let path = format!(
                "/sys/fs/bpf/mullvad-exclude-{}-{}",
                program.name().to_string_lossy(),
                cgroup.inode()
            );

            // We could do program.attach_cgroup() now, but then the program will be detached
            // and unloaded when this process exits. To work around this, we temporarily "pin"
            // the program to a file, and attach it to the cgroup.
            //
            // program.attach_cgroup(cgroup.fd.as_raw_fd());

            // "Pin" eBPF program to a file in /sys/fs/bpf/
            program
                .pin(&path)
                .with_context(|| anyhow!("Failed to pin eBPF program {:?}", program.name()))?;

            let attach_type = program.attach_type();

            // Get a file descriptor to the pinned file.
            let program = Program::fd_from_pinned_path(&path)?;

            // Attach the program to the excluded cgroup.
            // SAFETY:
            // - `OwnedFd` and `BorrowedFd` are always valid file descriptors.
            // - bpf_prog_attach is trivially safe to call.
            let code = unsafe {
                bpf_prog_attach(
                    program.as_raw_fd(),
                    cgroup.fd().as_raw_fd(),
                    attach_type as bpf_attach_type,
                    0,
                )
            };
            if code != 0 {
                return Err(io::Error::last_os_error())
                    .context("bpf_prog_attach returned error")
                    .with_context(|| {
                        anyhow!(
                            "Failed to attach eBPF program to cgroup at {:?}",
                            cgroup.path()
                        )
                    });
            }

            // We can now safely remove the pinned eBPF file.
            // The program will persist until the cgroup is removed.
            remove_file(&path)
                .with_context(|| anyhow!("Failed to clean up temporary eBPF file at {path:?}"))?;
        }

        Ok(())
    }
    fn exclude_current_cgroup() -> anyhow::Result<()> {
        let cgroup = get_current_cgroup().context("Failed to get current cgroup")?;

        install_exclusion_bpf_for_cgroup(&cgroup).with_context(|| {
            anyhow!(
                "Failed to install mullvad-exclude eBPF into cgroup {:?}",
                cgroup.name()
            )
        })?;

        Ok(())
    }

    fn run() -> anyhow::Result<()> {
        let args_os: Vec<OsString> = args_os().skip(1).collect();
        let flags: Vec<&str> = args_os
            .iter()
            .map_while(|arg| arg.to_str())
            .take_while(|arg| arg.starts_with("-"))
            .collect();
        let command: Vec<OsString> = args_os.iter().skip(flags.len()).cloned().collect();

        for flag in flags {
            match flag {
                "-h" | "--help" => return print_usage(None),
                f => return print_usage(Some(f)),
            }
        }

        let real_uid = getuid();
        let real_gid = getgid();

        let args: Vec<_> = command
            .into_iter()
            .map(OsString::into_vec)
            .map(CString::new)
            .collect::<Result<_, _>>()
            .context("Argument contains nul byte")?;

        let [program, ..] = &args[..] else {
            bail!("No command specified");
        };

        // Not strictly necessary, but temporarily drop privileges before interacting with D-Bus
        seteuid(real_uid).context("Failed to drop EUID")?;
        setegid(real_gid).context("Failed to drop EGID")?;

        systemd::join_scope_unit(real_uid.is_root())
            .context("Failed to join systemd scope unit")?;

        seteuid(0.into()).context("Failed to regain root EUID")?;
        setegid(0.into()).context("Failed to regain root EGID")?;

        exclude_current_cgroup()?;

        setuid(real_uid).context("Failed to drop UID")?;
        setgid(real_gid).context("Failed to drop GID")?;

        let Err(e) = execvp(&program, &args);
        eprintln!("Failed to exec {program:?}: {e}");
        std::process::exit(e as i32)
    }

    fn print_usage(invalid_arg: Option<&str>) -> Result<(), anyhow::Error> {
        println!("{}", include_str!("../usage.txt"));

        if let Some(arg) = invalid_arg {
            bail!("Invalid argument: {arg:?}");
        }

        Ok(())
    }

    mod systemd {
        use anyhow::{Context, bail};
        use zbus::{
            MatchRule,
            blocking::{Connection, MessageIterator},
            zvariant::{OwnedObjectPath, OwnedValue, Value},
        };

        // TODO: Document
        #[zbus::proxy(
            interface = "org.freedesktop.systemd1.Manager",
            default_service = "org.freedesktop.systemd1",
            default_path = "/org/freedesktop/systemd1"
        )]
        trait SystemdManager {
            fn start_transient_unit(
                &self,
                name: &str,
                mode: &str,
                properties: Vec<(&str, Value<'_>)>,
                aux: Vec<(String, Vec<(String, OwnedValue)>)>,
            ) -> zbus::Result<OwnedObjectPath>;
        }

        /// Create and join new scope unit in systemd for the current process. This also moves it
        /// into a new cgroup.
        ///
        /// This is approximately equivalent to `systemd-run --scope [--user] ...`, except that it
        /// applies to the current process.
        ///
        /// References:
        /// - system-run: https://github.com/systemd/systemd/blob/f76f0f99354b0485e3e13c2608bc26f969312687/src/run/run.c#L1671-L1699
        /// - man org.freedesktop.systemd1 - https://www.man7.org/linux/man-pages/man5/org.freedesktop.systemd1.5.html
        pub fn join_scope_unit(is_root: bool) -> anyhow::Result<()> {
            let connection = if is_root {
                Connection::system().context("Failed to connect to system bus")?
            } else {
                Connection::session().context("Failed to connect to user/session bus")?
            };

            // Create a match rule to listen for JobRemoved() signals
            // Must be done before calling StartTransientUnit().
            // TODO: Not sure if fine to wait before calling `next`. See docs on MessageIterator.
            let rule = MatchRule::builder()
                .sender("org.freedesktop.systemd1")?
                .interface("org.freedesktop.systemd1.Manager")?
                .member("JobRemoved")?
                .build();
            let mut msg_iter = MessageIterator::for_match_rule(rule, &connection, None)
                .context("Failed to create message iterator")?;

            // Now create the scope unit by calling StartTransientUnit()
            let proxy = SystemdManagerProxyBlocking::new(&connection)
                .context("Failed to create proxy to systemd manager")?;

            let properties = vec![
                // systemd will move these processes into the new scope/cgroup.
                // We only want to move the current process.
                ("PIDs", Value::new(vec![std::process::id()])),
                // Pin the unit(?).
                // TODO: Not sure what this actually adds. Taken from systemd-run:
                // https://github.com/systemd/systemd/blob/f76f0f99354b0485e3e13c2608bc26f969312687/src/run/run.c#L1671-L1699
                // https://github.com/systemd/systemd/blob/f76f0f99354b0485e3e13c2608bc26f969312687/src/run/run.c#L1346-L1350
                ("AddRef", Value::Bool(true)),
            ];

            let job_path = proxy
                .start_transient_unit(
                    // TODO: pid might be too likely to collide
                    &format!("mullvad-exclude-{}.scope", std::process::id()),
                    "fail",
                    properties,
                    vec![],
                )
                .context("StartTransientUnit failed")?;

            // StartTransientUnit() returns a path to a job object. We can wait for its JobRemoved()
            // signal to know when it is done.
            while let Some(msg) = msg_iter.next() {
                // Return value: job ID, bus path, primary unit name, result
                let (job_id, bus_path, unit_name, result): (u32, OwnedObjectPath, String, String) =
                    msg.context("Failed to get D-Bus message")?
                        .body()
                        .deserialize()
                        .context("Failed to deserialize JobRemoved() message")?;

                if bus_path == job_path {
                    if result != "done" {
                        bail!(
                            "systemd job {job_id} did not complete successfully for scope {unit_name}: {result}"
                        );
                    }
                    break;
                }
            }

            Ok(())
        }
    }
}
