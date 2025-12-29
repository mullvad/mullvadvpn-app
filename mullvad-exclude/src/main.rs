fn main() {
    #[cfg(target_os = "linux")]
    inner::main();
}

#[cfg(target_os = "linux")]
mod inner {
    use anyhow::{Context, anyhow, bail};
    use nftnl::{Batch, Chain, FinalizedBatch, ProtoFamily, Rule, Table, nft_expr};
    use nix::unistd::{execvp, getgid, getuid, setgid, setuid};
    use std::{
        env::args_os,
        ffi::{CStr, CString, OsString},
        fmt::Write as _,
        os::unix::ffi::OsStringExt,
        path::Path,
        process::Command,
    };
    use talpid_cgroup::v2::CGroup2;

    // TODO: comment
    pub fn main() {
        let Err(e) = run() else {
            return;
        };

        let mut s = format!("Error: {e}");
        let mut source = e.source();
        while let Some(error) = source {
            write!(&mut s, "\nCaused by: {error}").expect("formatting failed");
            source = error.source();
        }
        eprintln!("{s}");

        std::process::exit(1);
    }

    fn get_current_cgroup() -> anyhow::Result<(CGroup2, u32)> {
        let cgroup_file = std::fs::read_to_string("/proc/self/cgroup")
            .context("Failed to read /proc/self/cgroup")?;

        // /proc/self/cgroup contains a line that looks like this:
        // 0::/user.slice/user-1000.slice/user@1000.service/app.slice/app-launcher-appname-1234.scope
        let cgroup_path = cgroup_file
            .lines()
            .filter_map(|line| line.strip_prefix("0::/"))
            .next()
            .context("Expected a line starting with '0::/' containing the cgroup path")
            .context("Failed to parse /proc/self/cgroup")?
            .trim();
        let cgroup_path = Path::new(cgroup_path);
        let cgroup_depth = cgroup_path.components().count() as u32;
        let cgroup_fs_path = Path::new("/sys/fs/cgroup").join(cgroup_path);
        let cgroup = CGroup2::open(cgroup_fs_path).context("Failed to open cgroup")?;

        Ok((cgroup, cgroup_depth))
    }

    fn add_nft_table(cgroup: &CGroup2, cgroup_depth: u32) -> anyhow::Result<CString> {
        let mut batch = Batch::new();
        let cgroup_name = cgroup
            .name()
            .to_str()
            .context("cgroup name must be utf-8")?;
        let table_name = CString::new(format!("mullvad-exclude-{cgroup_name}",))
            .context("Invalid table name")?;
        let table = Table::new(&table_name, ProtoFamily::Inet);
        batch.add(&table, nftnl::MsgType::Add);
        let mut out_chain = Chain::new(c"chain", &table);
        out_chain.set_type(nftnl::ChainType::Route);
        out_chain.set_hook(nftnl::Hook::Out, nix::libc::NF_IP_PRI_MANGLE);
        out_chain.set_policy(nftnl::Policy::Accept);
        batch.add(&out_chain, nftnl::MsgType::Add);

        // === ADD CGROUPV2 RULE  ===
        let mut rule = Rule::new(&out_chain);
        rule.add_expr(&nft_expr!(socket cgroupv2 level cgroup_depth));
        rule.add_expr(&nft_expr!(cmp == cgroup.inode()));
        pub const MARK: u32 = 0xf41;
        pub const FWMARK: u32 = 0x6d6f6c65;
        rule.add_expr(&nft_expr!(immediate data MARK));
        rule.add_expr(&nft_expr!(ct mark set));
        rule.add_expr(&nft_expr!(immediate data FWMARK));
        rule.add_expr(&nft_expr!(meta mark set));
        // rule.add_expr(&nft_expr!(counter));
        batch.add(&rule, nftnl::MsgType::Add);

        let finalized_batch = batch.finalize();
        send_and_process(&finalized_batch)?;
        Ok(table_name)
    }

    // TODO: clean up nft table after program exits
    fn del_nft_table(table_name: &CStr) -> anyhow::Result<()> {
        let mut batch = Batch::new();
        let table = Table::new(table_name, ProtoFamily::Inet);
        batch.add(&table, nftnl::MsgType::Del);
        let finalized_batch = batch.finalize();
        send_and_process(&finalized_batch)?;
        Ok(())
    }

    fn send_and_process(batch: &FinalizedBatch) -> anyhow::Result<()> {
        // Create a netlink socket to netfilter.
        let socket = mnl::Socket::new(mnl::Bus::Netfilter)?;
        let portid = socket.portid();

        // Send all the bytes in the batch.
        socket.send_all(batch)?;

        // TODO: this buffer must be aligned to nlmsghdr
        let mut buffer = vec![0; nftnl::nft_nlmsg_maxsize() as usize];
        let mut expected_seqs = batch.sequence_numbers();

        // Process acknowledgment messages from netfilter.
        while !expected_seqs.is_empty() {
            for message in socket.recv(&mut buffer[..])? {
                let message = message?;
                let expected_seq = expected_seqs.next().expect("Unexpected ACK");
                // Validate sequence number and check for error messages
                mnl::cb_run(message, expected_seq, portid)?;
            }
        }
        Ok(())
    }

    fn exclude_current_cgroup() -> anyhow::Result<()> {
        let (cgroup, depth) = get_current_cgroup().context("Failed to get current cgroup")?;
        add_nft_table(&cgroup, depth)?;
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

        let mut current_cgroup = false;
        for flag in flags {
            match flag {
                "-h" | "--help" => return print_usage(None),
                "--current-cgroup" => current_cgroup = true,
                f => return print_usage(Some(f)),
            }
        }

        let real_uid = getuid();
        let real_gid = getgid();

        if current_cgroup {
            let args: Vec<_> = command
                .into_iter()
                .map(OsString::into_vec)
                .map(CString::new)
                .collect::<Result<_, _>>()
                .context("Argument contains nul byte")?;

            let [program, ..] = &args[..] else {
                bail!("No command specified");
            };

            exclude_current_cgroup()?;

            setuid(real_uid).context("Failed to drop UID")?;
            setgid(real_gid).context("Failed to drop GID")?;

            let Err(e) = execvp(&program, &args);
            eprintln!("Failed to exec {program:?}: {e}");
            std::process::exit(e as i32)
        } else {
            setuid(real_uid).context("Failed to drop UID")?;
            setgid(real_gid).context("Failed to drop GID")?;

            let [program, args @ ..] = &command[..] else {
                bail!("No command specified");
            };

            let is_not_root = !real_uid.is_root();

            let status = Command::new("/usr/bin/systemd-run")
                .args(is_not_root.then_some("--user"))
                .arg("--scope")
                .args(["mullvad-exclude", "--current-cgroup"])
                .arg(program)
                .args(args)
                .spawn()
                .with_context(|| anyhow!("Failed to spawn {program:?}"))?
                .wait()
                .expect("wait failed");

            if !status.success() {
                bail!("program errored");
            }
        }

        Ok(())
    }

    fn print_usage(invalid_arg: Option<&str>) -> Result<(), anyhow::Error> {
        println!("{}", include_str!("../usage.txt"));

        if let Some(arg) = invalid_arg {
            bail!("Invalid argument: {arg:?}");
        }

        Ok(())
    }
}
