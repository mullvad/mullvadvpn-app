import { TunnelState } from './daemon-rpc-types';

export function connectEnabled(
  connectedToDaemon: boolean,
  loggedIn: boolean,
  tunnelState: TunnelState['state'],
) {
  return (
    connectedToDaemon &&
    loggedIn &&
    (tunnelState === 'disconnected' || tunnelState === 'disconnecting' || tunnelState === 'error')
  );
}

export function reconnectEnabled(
  connectedToDaemon: boolean,
  loggedIn: boolean,
  tunnelState: TunnelState['state'],
) {
  return (
    connectedToDaemon &&
    loggedIn &&
    (tunnelState === 'connected' || tunnelState === 'connecting' || tunnelState === 'error')
  );
}

// Disconnecting while logged out is allowed since it's possible to "connect" and end up in the
// blocked state with the CLI.
export function disconnectEnabled(connectedToDaemon: boolean, tunnelState: TunnelState['state']) {
  return (
    connectedToDaemon &&
    (tunnelState === 'connected' || tunnelState === 'connecting' || tunnelState === 'error')
  );
}
