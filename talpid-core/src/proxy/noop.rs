use std::sync::mpsc;

use super::{ProxyMonitor, ProxyMonitorCloseHandle, Result, WaitResult};

pub struct NoopProxyMonitor {
    tx: mpsc::Sender<()>,
    rx: mpsc::Receiver<()>,
    port: u16,
}

impl NoopProxyMonitor {
    pub fn start(port: u16) -> Result<Self> {
        let (tx, rx) = mpsc::channel();
        Ok(NoopProxyMonitor { tx, rx, port })
    }
}

impl ProxyMonitor for NoopProxyMonitor {
    fn close_handle(&mut self) -> Box<dyn ProxyMonitorCloseHandle> {
        Box::new(NoopProxyMonitorCloseHandle {
            tx: self.tx.clone(),
        })
    }

    fn wait(self: Box<Self>) -> Result<WaitResult> {
        let _ = self.rx.recv();
        Ok(WaitResult::ProperShutdown)
    }

    fn port(&self) -> u16 {
        self.port
    }
}

struct NoopProxyMonitorCloseHandle {
    tx: mpsc::Sender<()>,
}

impl ProxyMonitorCloseHandle for NoopProxyMonitorCloseHandle {
    fn close(self: Box<Self>) -> Result<()> {
        let _ = self.tx.send(());
        Ok(())
    }
}
