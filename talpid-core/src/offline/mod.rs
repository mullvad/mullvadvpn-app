use futures::channel::mpsc::UnboundedSender;
use once_cell::sync::Lazy;
#[cfg(not(target_os = "android"))]
use talpid_routing::RouteManagerHandle;
#[cfg(target_os = "android")]
use talpid_types::android::AndroidContext;
use talpid_types::net::Connectivity;

#[cfg(target_os = "macos")]
#[path = "macos.rs"]
mod imp;

#[cfg(target_os = "windows")]
#[path = "windows.rs"]
mod imp;

#[cfg(target_os = "linux")]
#[path = "linux.rs"]
mod imp;

#[cfg(target_os = "android")]
#[path = "android.rs"]
mod imp;

/// Disables offline monitor
static FORCE_DISABLE_OFFLINE_MONITOR: Lazy<bool> = Lazy::new(|| {
    std::env::var("TALPID_DISABLE_OFFLINE_MONITOR")
        .map(|v| v != "0")
        .unwrap_or(false)
});

pub use self::imp::Error;

pub struct MonitorHandle(Option<imp::MonitorHandle>);

impl MonitorHandle {
    pub async fn connectivity(&self) -> Connectivity {
        match self.0.as_ref() {
            Some(monitor) => monitor.connectivity().await,
            None => Connectivity::Unknown,
        }
    }
}

#[cfg(not(target_os = "android"))]
pub async fn spawn_monitor(
    sender: UnboundedSender<Connectivity>,
    route_manager: RouteManagerHandle,
    #[cfg(target_os = "linux")] fwmark: Option<u32>,
) -> Result<MonitorHandle, Error> {
    let monitor = if *FORCE_DISABLE_OFFLINE_MONITOR {
        None
    } else {
        Some(
            imp::spawn_monitor(
                sender,
                route_manager,
                #[cfg(target_os = "linux")]
                fwmark,
            )
            .await?,
        )
    };

    Ok(MonitorHandle(monitor))
}

#[cfg(target_os = "android")]
pub async fn spawn_monitor(
    sender: UnboundedSender<Connectivity>,
    android_context: AndroidContext,
) -> Result<MonitorHandle, Error> {
    let monitor = if *FORCE_DISABLE_OFFLINE_MONITOR {
        None
    } else {
        let tx = forward_map(sender, |b| Connectivity::Status { ipv6: b });
        Some(imp::spawn_monitor(tx, android_context).await?)
    };

    Ok(MonitorHandle(monitor))
}

/// Map one kind of [`UnboundedSender<A>`] into an [`UnboundedSender<B>`] via
/// `f`.
///
/// This is accomplished by spawning a new [`tokio::task`] which listens on the
/// receiving end of an unbounded mpsc channel of type `B`. Whenever it receives
/// a new element of type `B`, it is mapped to an element of type `A` and
/// instantly forwarded on the original channel `a_tx`.
#[cfg(target_os = "android")]
fn forward_map<A, B, F>(mut a_tx: UnboundedSender<A>, f: F) -> UnboundedSender<B>
where
    A: Send + 'static,
    B: Send + 'static,
    F: Fn(B) -> A + Send + 'static,
{
    use futures::{SinkExt, StreamExt};
    let (b_tx, mut b_rx) = futures::channel::mpsc::unbounded::<B>();
    tokio::spawn(async move {
        while let Some(b) = b_rx.next().await {
            let _ = a_tx.send(f(b)).await;
        }
    });

    b_tx
}
