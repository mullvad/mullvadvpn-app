use async_trait::async_trait;
use futures::{channel::mpsc, StreamExt};

use super::{ProxyMonitor, ProxyMonitorCloseHandle, Result};

pub struct NoopProxyMonitor {
    tx: mpsc::UnboundedSender<()>,
    rx: mpsc::UnboundedReceiver<()>,
    port: u16,
}

impl NoopProxyMonitor {
    pub fn start(port: u16) -> Result<Self> {
        let (tx, rx) = mpsc::unbounded();
        Ok(NoopProxyMonitor { tx, rx, port })
    }
}

#[async_trait]
impl ProxyMonitor for NoopProxyMonitor {
    fn close_handle(&mut self) -> Box<dyn ProxyMonitorCloseHandle> {
        Box::new(NoopProxyMonitorCloseHandle {
            tx: self.tx.clone(),
        })
    }

    async fn wait(mut self: Box<Self>) -> Result<()> {
        let _ = self.rx.next().await;
        Ok(())
    }

    fn port(&self) -> u16 {
        self.port
    }
}

struct NoopProxyMonitorCloseHandle {
    tx: mpsc::UnboundedSender<()>,
}

impl ProxyMonitorCloseHandle for NoopProxyMonitorCloseHandle {
    fn close(self: Box<Self>) -> Result<()> {
        let _ = self.tx.unbounded_send(());
        Ok(())
    }
}
