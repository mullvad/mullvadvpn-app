// @flow

export type DaemonConnectedAction = {
  type: 'DAEMON_CONNECTED',
};
export type DaemonDisconnectedAction = {
  type: 'DAEMON_DISCONNECTED',
};

export type DaemonAction = DaemonConnectedAction | DaemonDisconnectedAction;

function connected(): DaemonConnectedAction {
  return {
    type: 'DAEMON_CONNECTED',
  };
}

function disconnected(): DaemonDisconnectedAction {
  return {
    type: 'DAEMON_DISCONNECTED',
  };
}

export default { connected, disconnected };
