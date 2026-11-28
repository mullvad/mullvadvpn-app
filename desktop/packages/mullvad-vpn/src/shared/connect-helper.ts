import { TunnelState } from './daemon-rpc-types';

export function connectEnabled(
  connectedToDaemon: boolean,
  hasIdentity: boolean,
  tunnelState: TunnelState['state'],
) {
  return (
    connectedToDaemon &&
    hasIdentity &&
    (tunnelState === 'disconnected' || tunnelState === 'disconnecting' || tunnelState === 'error')
  );
}

export function reconnectEnabled(
  connectedToDaemon: boolean,
  hasIdentity: boolean,
  tunnelState: TunnelState['state'],
) {
  return (
    connectedToDaemon &&
    hasIdentity &&
    (tunnelState === 'connected' || tunnelState === 'connecting' || tunnelState === 'error')
  );
}

// Disconnecting without a config is allowed since it's possible to end up in the blocked state
// with the CLI.
export function disconnectEnabled(connectedToDaemon: boolean, tunnelState: TunnelState['state']) {
  return (
    connectedToDaemon &&
    (tunnelState === 'connected' || tunnelState === 'connecting' || tunnelState === 'error')
  );
}
