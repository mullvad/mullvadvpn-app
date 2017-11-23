// @flow

import path from 'path';
import React from 'react';
import ReactDOM from 'react-dom';
import { Provider } from 'react-redux';
import { ConnectedRouter } from 'react-router-redux';
import { createMemoryHistory } from 'history';
import { webFrame, ipcRenderer } from 'electron';
import log from 'electron-log';
import makeRoutes from './routes';
import configureStore from './redux/store';
import { Backend } from './lib/backend';

import type { ConnectionState } from './redux/connection/reducers';
import type { TrayIconType } from './lib/tray-icon-manager';

const initialState = null;
const memoryHistory = createMemoryHistory();
const store = configureStore(initialState, memoryHistory);

//////////////////////////////////////////////////////////////////////////
// Backend
//////////////////////////////////////////////////////////////////////////
const backend = new Backend(store);
ipcRenderer.on('backend-info', (_event, args) => {
  backend.setCredentials(args.credentials);
  backend.sync();
  backend.autologin()
    .then( () => {
      return backend.syncRelaySettings();
    })
    .then( () => {
      const { settings: { relaySettings: { host, protocol, port } } } = store.getState();

      return backend.connect(host, protocol, port);
    })
    .catch( e => {
      if (e.type === 'NO_ACCOUNT') {
        log.debug('No user set in the backend, showing window');
        ipcRenderer.send('show-window');
      }
    });
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

if(navigator.serviceWorker) {
  navigator.serviceWorker.register(path.join(__dirname, 'tilecache.sw.js'))
    .then((registration) => {
      log.info('ServiceWorker registration successful with scope: ', registration.scope);
    }).catch((err) => {
      log.info('ServiceWorker registration failed: ', err);
    });
}

ipcRenderer.send('on-browser-window-ready');

function getRootElement() {
  const currentScript = document.currentScript;
  if (!currentScript) {
    throw new Error('Missing document.currentScript');
  }

  const containerId = currentScript.getAttribute('data-container');
  if(!containerId) {
    throw new Error('Missing data-container attribute.');
  }

  const rootElement = document.querySelector(containerId);
  if(!rootElement) {
    throw new Error('Missing root element.');
  }

  return rootElement;
}


ReactDOM.render(
  <Provider store={ store }>
    <ConnectedRouter history={ memoryHistory }>
      { makeRoutes(store.getState, { backend }) }
    </ConnectedRouter>
  </Provider>,
  getRootElement()
);
