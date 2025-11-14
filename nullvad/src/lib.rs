use std::{
    net::{Ipv4Addr, Ipv6Addr},
    os::fd::{AsFd, AsRawFd, OwnedFd},
    panic,
    path::Path,
};

use anyhow::{Context, anyhow, bail};
use futures::StreamExt;
use nix::{
    mount::{MntFlags, umount2},
    sched::{CloneFlags, setns},
    unistd::unlink,
};

use rtnetlink::{LinkUnspec, LinkVeth, RouteMessageBuilder};
use tokio::{fs, runtime, sync::oneshot, task::JoinSet};

pub mod nft;

pub type Error = anyhow::Error;
pub type Result<T> = anyhow::Result<T>;

// TODO `/var/run` or `/run`, which is more portable? will we need to check both?
const NETNS_DIR: &str = "/run/netns";
pub const NETNS_NAME: &str = "nullvad";
pub const VETH1: &str = "veth-mullvad";
pub const VETH2: &str = "veth-nullvad";
pub const VETH4_IPV4: Ipv4Addr = Ipv4Addr::new(172, 25, 1, 1);
pub const VETH2_IPV4: Ipv4Addr = Ipv4Addr::new(172, 25, 1, 2);
pub const VETH4_IPV6: Ipv6Addr = Ipv6Addr::from_bits(0x_f800_0000_0000_0000_0000_0000_0000_0000); // TODO
pub const VETH2_IPV6: Ipv6Addr = Ipv6Addr::from_bits(0x_f800_0000_0000_0000_0000_0000_0000_0000); // TODO

/// Create split-tunneling network namespace
pub async fn up() -> anyhow::Result<()> {
    let (connection, handle, _) = rtnetlink::new_connection()?;
    let mut tasks = JoinSet::new();
    tasks.spawn(connection); // poor-mans abort-on-drop

    // TODO: this function forks and does weird stuff. should we do it ourselves instead? would fix fd-race.
    rtnetlink::NetworkNamespace::add(NETNS_NAME.into())
        .await
        .context("Failed to create network namespace")?;

    // TODO: create drop-bomb that removes namespace in case of error

    // TODO: is this the correct fd?
    // TODO: this is racey
    let netns_fd = fs::OpenOptions::new()
        .read(true)
        .open(Path::new(NETNS_DIR).join(NETNS_NAME))
        .await?;

    // ip link add dev VETH1 type veth peer name VETH2
    handle
        .link()
        .add(LinkVeth::new(VETH1, VETH2).build())
        .execute()
        .await
        .with_context(|| anyhow!("failed to create veth-pair {VETH1} & {VETH2}"))?;

    // TODO: create drop-bomb that removes interfaces in case of error

    // TODO: this is racey, these might not return the interfaces we created
    // nor can we guarantee that they aren't removed while we work.
    let veth_1 = get_link_index(&handle, VETH1).await?;
    let veth_2 = get_link_index(&handle, VETH2).await?;

    // ip link set VETH2 netns NETNS_NAME
    handle
        .link()
        .set(
            LinkUnspec::new_with_name(VETH2)
                .setns_by_fd(netns_fd.as_fd().as_raw_fd())
                .build(),
        )
        .execute()
        .await
        .with_context(|| anyhow!("`ip link set {VETH1} nets {NETNS_NAME}` failed"))?;

    // ip addr add VETH4_IPV4/30 dev VETH1
    handle
        .address()
        .add(veth_1, Ipv4Addr::new(172, 25, 1, 1).into(), 30)
        .execute()
        .await
        .with_context(|| anyhow!("`ip addr add {VETH4_IPV4}/30 dev {VETH1}` failed"))?;

    nft::add_nft_rules().await?;

    // ip link set dev veth10 up
    handle
        .link()
        .set(LinkUnspec::new_with_name(VETH1).up().build())
        .execute()
        .await
        .with_context(|| anyhow!("`ip link set dev {VETH1} up` failed"))?;

    do_in_namespace_async(netns_fd.try_clone().await?, async move || {
        let (connection, handle, _) = rtnetlink::new_connection()?;
        let mut tasks = JoinSet::new();
        tasks.spawn(connection); // poor-mans abort-on-drop

        // ip link set dev lo up
        handle
            .link()
            .set(LinkUnspec::new_with_name("lo").up().build())
            .execute()
            .await
            .context("`ip link set dev lo up` failed")?;

        // ip addr add VETH2_IPV4/30 dev VETH2
        handle
            .address()
            .add(veth_2, VETH2_IPV4.into(), 30)
            .execute()
            .await
            .with_context(|| anyhow!("`ip addr add {VETH2_IPV4}/30 dev {VETH2}` failed"))?;

        // ip link set dev VETH2 up
        handle
            .link()
            .set(LinkUnspec::new_with_index(veth_2).up().build())
            .execute()
            .await
            .with_context(|| anyhow!("`ip link set {VETH2} up` failed"))?;

        // ip route add default via VETH4_IPV4
        handle
            .route()
            .add(
                RouteMessageBuilder::<Ipv4Addr>::new()
                    .gateway(VETH4_IPV4)
                    .build(),
            )
            .execute()
            .await
            .with_context(|| anyhow!("`ip route add default via {VETH4_IPV4}` failed"))?;

        anyhow::Ok(())
    })
    .await??;

    // TODO: it worked! defuse drop-bombs

    Ok(())
}

/// Destroy split-tunneling network namespace
// TODO: make an async version?
pub fn down() {
    if let Err(e) = destroy_namespace() {
        log::error!("failed to destroy network namespace {NETNS_NAME}: {e:#?}");
    }

    if let Err(e) = nft::remove_nft_rules() {
        log::error!("{e:#?}");
    }
}

/// Destroy the network namespace by unmounting and removing the persistent namespace file.
// TODO: is it possible that the namespace still persists? if so, document.
pub fn destroy_namespace() -> anyhow::Result<()> {
    let netns_path = Path::new(NETNS_DIR).join(NETNS_NAME);
    (umount2(&netns_path, MntFlags::MNT_DETACH).context("Unmount failed"))
        .and_then(|_| unlink(&netns_path).context("Failed to remove file"))
        .with_context(|| anyhow!("Failed to destroy network namespace {NETNS_NAME:?}"))
}

/// Get the index of a linux network link.
async fn get_link_index(handle: &rtnetlink::Handle, name: &str) -> anyhow::Result<u32> {
    let links = handle.link().get().match_name(name.into()).execute();
    let mut veth_2: Vec<_> = links.collect().await;

    if veth_2.len() > 1 {
        bail!("multiple links called {name}");
    }

    let Some(veth_2) = veth_2.pop() else {
        bail!("no link called {name}");
    };

    Ok(veth_2?.header.index)
}

pub async fn open_namespace_file() -> anyhow::Result<OwnedFd> {
    // TODO: validate that this file is actually a namespace?
    let netns_fd = fs::OpenOptions::new()
        .read(true)
        .open(Path::new(NETNS_DIR).join(NETNS_NAME))
        .await
        .context("Failed to open namespace file")?;

    Ok(netns_fd.into_std().await.into())
}

/// Execute `f` on a thread running in the network namespace referenced by `namespace_fd`.
///
/// `f` will be executed on a new tokio runtime, running on a dedicated thread.
///
/// See also: [do_in_namespace].
pub async fn do_in_namespace_async<Fd, Fn, Ft, T>(namespace_fd: Fd, f: Fn) -> anyhow::Result<T>
where
    Fd: AsFd,
    Fn: FnOnce() -> Ft,
    Ft: Future<Output = T>,
    // ---
    Fd: Send + 'static,
    Fn: Send + 'static,
    T: Send + 'static,
{
    do_in_namespace(namespace_fd, move || {
        // We don't want to risk tasks being executed on worker threads in a different namespace,
        // so we create an isolated runtime within this namespace to run tasks on.
        runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("Failed to create tokio runtime")
            .block_on(f())
    })
    .await
}

/// Execute `f` on a thread running in the network namespace referenced by `namespace_fd`.
///
/// `namespace_fd` should be a file in `/run/netns/`.
///
/// Any threads created by `f` will be executed in that same namespace, but be careful not to
/// leak execution onto other threads, i.e. by spawning them on an existing async runtime.
///
/// See also: [do_in_namespace_async].
pub async fn do_in_namespace<Fd, Fn, T>(namespace_fd: Fd, f: Fn) -> anyhow::Result<T>
where
    Fd: AsFd,
    Fn: FnOnce() -> T,
    // ---
    Fd: Send + 'static,
    Fn: Send + 'static,
    T: Send + 'static,
{
    let (tx, rx) = oneshot::channel();

    let thread = std::thread::spawn(move || {
        let t = setns(namespace_fd, CloneFlags::empty())
            .with_context(|| anyhow!("Failed to enter namespace {NETNS_NAME}"))
            .map(|()| f());
        let _ = tx.send(t);
    });

    let t = rx.await;

    match t {
        Ok(t) => t,
        Err(_dropped) => {
            // thread must have panicked
            let err = thread.join().unwrap_err();
            panic::resume_unwind(err)
        }
    }
}
