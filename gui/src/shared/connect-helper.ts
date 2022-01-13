import { AccountToken, TunnelState } from './daemon-rpc-types';

export function connectEnabled(
  connectedToDaemon: boolean,
  accountToken: AccountToken | undefined,
  tunnelState: TunnelState['state'],
) {
  return (
    connectedToDaemon &&
    accountToken !== undefined &&
    (tunnelState === 'disconnected' || tunnelState === 'disconnecting' || tunnelState === 'error')
  );
}

export function reconnectEnabled(
  connectedToDaemon: boolean,
  accountToken: AccountToken | undefined,
  tunnelState: TunnelState['state'],
) {
  return (
    connectedToDaemon &&
    accountToken !== undefined &&
    (tunnelState === 'connected' || tunnelState === 'connecting')
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
