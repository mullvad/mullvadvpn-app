// @flow

import React from 'react';
import { Component } from 'reactxp';
import { Provider } from 'react-redux';
import { ConnectedRouter } from 'react-router-redux';
import { createMemoryHistory } from 'history';
import { webFrame, ipcRenderer } from 'electron';
import { log } from './lib/platform';
import makeRoutes from './routes';
import configureStore from './redux/store';
import { Backend } from './lib/backend';
import { DaemonRpc } from './lib/daemon-rpc';
import { setShutdownHandler } from './shutdown-handler';

import type { ConnectionState } from './redux/connection/reducers';
import type { TrayIconType } from './tray-icon-controller';
import type { RpcCredentialsProvider, RpcCredentials } from './lib/backend';

const initialState = null;
const memoryHistory = createMemoryHistory();
const store = configureStore(initialState, memoryHistory);

class CredentialsProvider implements RpcCredentialsProvider {
  request(): Promise<RpcCredentials> {
    return new Promise((resolve, _reject) => {
      ipcRenderer.once('daemon-connection-ready', (_event, credentials: RpcCredentials) => {
        resolve(credentials);
      });
      ipcRenderer.send('discover-daemon-connection');
    });
  }
}

const rpc = new DaemonRpc();
const credentialsProvider = new CredentialsProvider();
const backend = new Backend(store, rpc, credentialsProvider);
backend.connect();

setShutdownHandler(async () => {
  log.info('Executing a shutdown handler');

  try {
    await backend.disconnectTunnel();
    log.info('Disconnected the tunnel');
  } catch (e) {
    log.error(`Failed to shutdown tunnel: ${e.message}`);
  }
});

/**
 * Get tray icon type based on connection state
 */
const getIconType = (s: ConnectionState): TrayIconType => {
  switch (s) {
    case 'connected':
      return 'secured';
    case 'connecting':
      return 'securing';
    default:
      return 'unsecured';
  }
};

/**
 * Update tray icon via IPC call
 */
const updateTrayIcon = () => {
  const { connection } = store.getState();

  // TODO: Only update the tray icon if the connection status changed
  ipcRenderer.send('change-tray-icon', getIconType(connection.status));
};

store.subscribe(updateTrayIcon);

// force update tray
updateTrayIcon();

// disable smart pinch.
webFrame.setVisualZoomLevelLimits(1, 1);

export default class App extends Component {
  render() {
    return (
      <Provider store={store}>
        <ConnectedRouter history={memoryHistory}>
          {makeRoutes(store.getState, { backend })}
        </ConnectedRouter>
      </Provider>
    );
  }
}
