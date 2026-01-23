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
    use libc::{AF_INET, AF_INET6};
    use libseccomp::{
        ScmpAction, ScmpArgCompare, ScmpCompareOp, ScmpFilterContext, ScmpNotifReq, ScmpNotifResp,
        ScmpNotifRespFlags, ScmpSyscall, error::SeccompErrno,
    };

    use nix::{
        cmsg_space,
        sys::socket::{ControlMessage, ControlMessageOwned, MsgFlags, recvmsg, sendmsg},
        unistd::{
            ForkResult, Pid, execvp, fork, getgid, getpid, getuid, setegid, seteuid, setgid, setuid,
        },
    };
    use std::{
        env::args_os,
        ffi::{CString, OsString},
        fs::remove_file,
        io::{self, IoSlice, IoSliceMut},
        os::{
            fd::{AsRawFd, FromRawFd, OwnedFd, RawFd},
            unix::{ffi::OsStringExt as _, net::UnixStream},
        },
        path::Path,
    };
    use talpid_cgroup::{SPLIT_TUNNEL_CGROUP_NAME, find_net_cls_mount, v1::CGroup1, v2::CGroup2};

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
    #[allow(dead_code)]
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
    #[allow(dead_code)]
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

    #[allow(dead_code)]
    fn add_to_cgroups_v1(pid: Pid) -> anyhow::Result<()> {
        let net_cls_dir = find_net_cls_mount()
            .context("Failed to find net_cls mount")?
            .context("No net_cls mount found")?;

        let cgroup_path = net_cls_dir.join(SPLIT_TUNNEL_CGROUP_NAME);

        CGroup1::open(cgroup_path)
            .and_then(|cgroup| cgroup.add_pid(pid))
            .context("Failed to add process to net_cls cgroup")
    }

    fn run() -> anyhow::Result<()> {
        let args_os: Vec<OsString> = args_os().skip(1).collect();
        let flags: Vec<&str> = args_os
            .iter()
            .map_while(|arg| arg.to_str())
            .take_while(|arg| arg.starts_with("-"))
            .collect();
        let command: Vec<OsString> = args_os.iter().skip(flags.len()).cloned().collect();
        let mut enable_seccomp = false;

        for flag in flags {
            match flag {
                "-h" | "--help" => return print_usage(None),
                // NOTE: This has overhead since socket() calls are intercepted.
                // It also forces
                // TODO: detect when running a flatpak
                "--intercept" => enable_seccomp = true,
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

        if talpid_cgroup::is_systemd_managed() {
            seteuid(real_uid).context("Failed to drop root temporarily")?;
            setegid(real_gid).context("Failed to drop root temporarily")?;
            systemd::join_scope_unit(real_uid.is_root(), program)?;
            setuid(0.into()).context("Failed to regain root")?;
            setgid(0.into()).context("Failed to regain root")?;
        }

        if enable_seccomp {
            let (notify_fd_tx, notify_fd_rx) =
                UnixStream::pair().context("Failed to create unix socket")?;

            let parent_pid = getpid();

            match unsafe { fork() }.context("fork failed")? {
                ForkResult::Parent { child: _ } => {
                    drop(notify_fd_rx);

                    // TODO: check support?
                    //let api_version = libseccomp::get_api().context("Failed to get libseccomp API")?;
                    let mut ctx = ScmpFilterContext::new(ScmpAction::Allow)
                        .context("Failed to create seccomp filter context")?;
                    // No need to set NO_NEW_PRIVS as we are sudo.
                    ctx.set_ctl_nnp(false)
                        .context("Failed to disable NO_NEW_PRIVS")?;
                    let socket_sys =
                        ScmpSyscall::from_name("socket").context("Failed to get socket syscall")?;
                    // The first socket() arg must be AF_INET or AF_INET6
                    // https://www.man7.org/linux/man-pages/man2/socket.2.html
                    let inet_cmp = ScmpArgCompare::new(
                        0,
                        ScmpCompareOp::Equal,
                        u64::try_from(AF_INET).unwrap(),
                    );
                    ctx.add_rule_conditional(ScmpAction::Notify, socket_sys.clone(), &[inet_cmp])
                        .context("Failed to add socket() AF_INET rule")?;
                    let inet6_cmp = ScmpArgCompare::new(
                        0,
                        ScmpCompareOp::Equal,
                        u64::try_from(AF_INET6).unwrap(),
                    );
                    ctx.add_rule_conditional(ScmpAction::Notify, socket_sys.clone(), &[inet6_cmp])
                        .context("Failed to add socket() AF_INET6 rule")?;

                    ctx.load().context("Failed to load seccomp filter")?;
                    let notify_fd = ctx.get_notify_fd().context("Failed to get notify fd")?;
                    // SAFETY: `notify_fd` is a valid file descriptor.
                    let notify_fd = unsafe { OwnedFd::from_raw_fd(notify_fd) };

                    // Send the seccomp notify fd to the parent process using SCM_RIGHTS.
                    // File descriptors are process-local, so we must use ancillary messages.
                    let fds = [notify_fd.as_raw_fd()];
                    let cmsg = [ControlMessage::ScmRights(&fds)];
                    // We need to send at least 1 byte of data along with the control message.
                    let iov = [IoSlice::new(&[0u8])];
                    sendmsg::<()>(
                        notify_fd_tx.as_raw_fd(),
                        &iov,
                        &cmsg,
                        MsgFlags::empty(),
                        None,
                    )
                    .context("Failed to send notify fd over socket")?;
                    drop(notify_fd_tx);
                    drop(notify_fd);

                    setgid(real_gid).context("setgid failed")?;
                    setuid(real_uid).context("setuid failed")?;

                    execvp(program, &args).context("execvp failed")?;
                    Ok(())
                }
                ForkResult::Child => {
                    drop(notify_fd_tx);

                    // Receive the seccomp notify fd from the parent process using SCM_RIGHTS.
                    let mut buf = [0u8; 1];
                    let mut iov = [IoSliceMut::new(&mut buf)];
                    let mut cmsg_buf = cmsg_space!(RawFd);
                    let msg = recvmsg::<()>(
                        notify_fd_rx.as_raw_fd(),
                        &mut iov,
                        Some(&mut cmsg_buf),
                        MsgFlags::empty(),
                    )
                    .context("Failed to receive notify fd from socket")?;

                    // Extract the file descriptor from the control message.
                    let notify_fd: RawFd = msg
                        .cmsgs()?
                        .find_map(|cmsg| match cmsg {
                            ControlMessageOwned::ScmRights(fds) => fds.first().copied(),
                            _ => None,
                        })
                        .context("Missing file descriptor in CMSG")?;

                    // SAFETY: The fd was just received via SCM_RIGHTS and is valid.
                    // The fd was produced by ScmpFilterContext::get_notify_fd.
                    // The fd does not need any cleanup other than close.
                    // Closing the fd is fine because ... TODO
                    let notify_fd = unsafe { OwnedFd::from_raw_fd(notify_fd) };
                    drop(notify_fd_rx);

                    // Run seccomp supervisor
                    seccomp_monitor(parent_pid, notify_fd)?;
                    Ok(())
                }
            }
        } else {
            // No seccomp; just exec the target program
            exclude_current_cgroup()?;
            setgid(real_gid).context("setgid failed")?;
            setuid(real_uid).context("setuid failed")?;
            execvp(program, &args).context("execvp failed")?;

            Ok(())
        }
    }

    /// Enable split tunneling for `supervised_pid` on the first monitored syscall from it or any child process.
    fn seccomp_monitor(supervised_pid: Pid, notify_fd: OwnedFd) -> anyhow::Result<()> {
        let mut handled_supervised_pid = false;

        loop {
            // Poll for seccomp notifications.
            // These trigger when a child process invokes `socket(AF_INET / AF_INET6)`.
            let req = match ScmpNotifReq::receive(notify_fd.as_raw_fd()) {
                Ok(req) => req,
                Err(err) => {
                    if err
                        .errno()
                        .map(|errno| errno == SeccompErrno::ECANCELED)
                        .unwrap_or(false)
                    {
                        // All children exited?
                        return Ok(());
                    }
                    bail!("ScmpNotifReq::receive failed: {err}");
                }
            };
            let handle_scmp_request = |req: ScmpNotifReq| -> anyhow::Result<ScmpNotifResp> {
                // Get the cgroup of the supervised process.
                // We can't assume the process is still in the cgroup we created for it,
                // since some programs (flatpak) create their own cgroup.
                if !handled_supervised_pid {
                    let cgroup_path = read_cgroup_path(supervised_pid.as_raw() as i32)
                        .context("Failed to read cgroup path")?;
                    eprintln!("Attaching to cgroup2 path: {}", cgroup_path);
                    // TODO: make sure we ALWAYS respond to thhe ScmpNotif.
                    let cgroup = CGroup2::open(format!("/sys/fs/cgroup{cgroup_path}"))
                        .context("Failed to open cgroup2")?;
                    install_exclusion_bpf_for_cgroup(&cgroup)
                        .context("Failed to install BPF hook for cgroup2")?;
                }

                Ok(ScmpNotifResp::new_continue(
                    req.id,
                    ScmpNotifRespFlags::empty(),
                ))
            };

            let resp = match handle_scmp_request(req) {
                Ok(resp) => {
                    handled_supervised_pid = true;
                    resp
                }
                Err(err) => {
                    eprintln!("handle_scmp_request failed: {err}");
                    ScmpNotifResp::new_error(req.id, -libc::EPERM, ScmpNotifRespFlags::empty())
                }
            };

            match resp.respond(notify_fd.as_raw_fd()) {
                Ok(_) => {}
                Err(err) => {
                    bail!("ScmpNotifResp::respond failed: {err}");
                }
            }
        }
    }

    fn read_cgroup_path(pid: i32) -> anyhow::Result<String> {
        let cgroup_content = std::fs::read_to_string(format!("/proc/{}/cgroup", pid))
            .context("Failed to read /proc/[pid]/cgroup")?;

        for line in cgroup_content.lines() {
            if let Some(path) = line.strip_prefix("0::") {
                return Ok(path.to_string());
            }
        }

        bail!("No cgroup v2 entry found in /proc/[pid]/cgroup")
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
        use rand::{Rng as _, distr::Alphanumeric};
        use std::ffi::CStr;
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
        #[allow(dead_code)]
        pub fn join_scope_unit(is_root: bool, program: &CStr) -> anyhow::Result<()> {
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

            // Generate a unique scope name according to systemds convention for DEs,
            // `app-<launcher>-<app_id>-<random>.scope`
            // See https://systemd.io/DESKTOP_ENVIRONMENTS/
            let mut rng = rand::rng();
            let random: String = (0..8).map(|_| rng.sample(Alphanumeric) as char).collect();
            let app_name = program_path_to_unit_name(program);
            let unit_name = format!("app-mullvadexclude-{app_name}-{random}.scope");

            let job_path = proxy
                .start_transient_unit(&unit_name, "fail", properties, vec![])
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

        /// Convert a program file path to an alphanumeric+underscores string.
        /// For example, "/bin/mullvad-exclude" becomes "mullvad_exclude"
        #[allow(dead_code)]
        fn program_path_to_unit_name(program: &CStr) -> String {
            let program = program.to_string_lossy();
            let (_path, file_name) = program.rsplit_once('/').unwrap_or(("", &program));
            file_name
                .chars()
                .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
                .collect()
        }
    }
}
