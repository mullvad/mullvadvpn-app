use super::super::{oneshot_send, CommandResult};
use crate::{Daemon, DaemonCommand, EventListener, ResponseTx};
use talpid_types::ErrorExt;

impl<L: EventListener> Daemon<L> {
    pub(in super::super) async fn handle_split_tunnel_command(
        &mut self,
        command: DaemonCommand,
    ) -> CommandResult {
        use DaemonCommand::*;
        match command {
            GetSplitTunnelProcesses(tx) => self.on_get_split_tunnel_processes(tx),
            AddSplitTunnelProcess(tx, pid) => self.on_add_split_tunnel_process(tx, pid),
            RemoveSplitTunnelProcess(tx, pid) => self.on_remove_split_tunnel_process(tx, pid),
            ClearSplitTunnelProcesses(tx) => self.on_clear_split_tunnel_processes(tx),
            cmd => return CommandResult::NotHandled(cmd),
        }
        CommandResult::Handled
    }

    fn on_get_split_tunnel_processes(
        &mut self,
        tx: ResponseTx<Vec<i32>, crate::split_tunnel::Error>,
    ) {
        let result = self.exclude_pids.list().map_err(|error| {
            log::error!("{}", error.display_chain_with_msg("Unable to obtain PIDs"));
            error
        });
        oneshot_send(tx, result, "get_split_tunnel_processes response");
    }

    fn on_add_split_tunnel_process(
        &mut self,
        tx: ResponseTx<(), crate::split_tunnel::Error>,
        pid: i32,
    ) {
        let result = self.exclude_pids.add(pid).map_err(|error| {
            log::error!("{}", error.display_chain_with_msg("Unable to add PID"));
            error
        });
        oneshot_send(tx, result, "add_split_tunnel_process response");
    }

    fn on_remove_split_tunnel_process(
        &mut self,
        tx: ResponseTx<(), crate::split_tunnel::Error>,
        pid: i32,
    ) {
        let result = self.exclude_pids.remove(pid).map_err(|error| {
            log::error!("{}", error.display_chain_with_msg("Unable to remove PID"));
            error
        });
        oneshot_send(tx, result, "remove_split_tunnel_process response");
    }

    fn on_clear_split_tunnel_processes(&mut self, tx: ResponseTx<(), crate::split_tunnel::Error>) {
        let result = self.exclude_pids.clear().map_err(|error| {
            log::error!("{}", error.display_chain_with_msg("Unable to clear PIDs"));
            error
        });
        oneshot_send(tx, result, "clear_split_tunnel_processes response");
    }
}
