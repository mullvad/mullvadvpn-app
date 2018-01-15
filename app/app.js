// @flow

import React from 'react';
import { Component} from 'reactxp';
import { Provider } from 'react-redux';
import { ConnectedRouter } from 'react-router-redux';
import { createMemoryHistory } from 'history';
import { webFrame, ipcRenderer } from 'electron';
import { log } from './lib/platform';
import makeRoutes from './routes';
import configureStore from './redux/store';
import { Backend, BackendError } from './lib/backend';

import type { ConnectionState } from './redux/connection/reducers';
import type { TrayIconType } from './lib/tray-icon-manager';

const initialState = null;
const memoryHistory = createMemoryHistory();
const store = configureStore(initialState, memoryHistory);

//////////////////////////////////////////////////////////////////////////
// Backend
//////////////////////////////////////////////////////////////////////////
const backend = new Backend(store);
ipcRenderer.on('backend-info', async (_event, args) => {
  backend.setCredentials(args.credentials);
  backend.sync();
  try {
    await backend.autologin();
    await backend.fetchRelaySettings();
    await backend.fetchSecurityState();
    await backend.connect();
  } catch (e) {
    if(e instanceof BackendError) {
      if(e.type === 'NO_ACCOUNT') {
        log.debug('No user set in the backend, showing window');
        ipcRenderer.send('show-window');
      }
    }
  }
});

ipcRenderer.on('shutdown', () => {
  log.info('Been told by the node process to shutdown');
  backend.shutdown()
    .catch( e => {
      log.warn('Unable to shut down the backend', e.message);
    });
});
//////////////////////////////////////////////////////////////////////////
//////////////////////////////////////////////////////////////////////////

//////////////////////////////////////////////////////////////////////////
// Tray icon
//////////////////////////////////////////////////////////////////////////

/**
 * Get tray icon type based on connection state
 */
const getIconType = (s: ConnectionState): TrayIconType => {
  switch(s) {
  case 'connected': return 'secured';
  case 'connecting': return 'securing';
  default: return 'unsecured';
  }
};

/**
 * Update tray icon via IPC call
 */
const updateTrayIcon = () => {
  const { connection } = store.getState();
  // TODO: Only update the tray icon if the connection status changed
  ipcRenderer.send('changeTrayIcon', getIconType(connection.status));
};
store.subscribe(updateTrayIcon);

// force update tray
updateTrayIcon();
//////////////////////////////////////////////////////////////////////////
//////////////////////////////////////////////////////////////////////////

// disable smart pinch.
webFrame.setZoomLevelLimits(1, 1);

ipcRenderer.send('on-browser-window-ready');


export default class App extends Component{
  render() {
    return (
      <Provider store={ store }>
        <ConnectedRouter history={ memoryHistory }>
          { makeRoutes(store.getState, { backend }) }
        </ConnectedRouter>
      </Provider>
    );
  }
}
