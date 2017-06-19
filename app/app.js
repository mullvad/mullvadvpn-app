import path from 'path';
import React from 'react';
import ReactDOM from 'react-dom';
import { Provider } from 'react-redux';
import { ConnectedRouter } from 'react-router-redux';
import { createMemoryHistory } from 'history';
import { webFrame, ipcRenderer } from 'electron';
import log from 'electron-log';
import makeRoutes from './routes';
import configureStore from './store';
import userActions from './actions/user';
import connectActions from './actions/connect';
import { Backend } from './lib/backend';
import mapBackendEventsToReduxActions from './lib/backend-redux-actions';
import mapBackendEventsToRouter from './lib/backend-routing';

import type { LoginState, ConnectionState } from './enums';
import type { TrayIconType } from './lib/tray-icon-manager';

const initialState = null;
const memoryHistory = createMemoryHistory();
const store = configureStore(initialState, memoryHistory);
const backend = new Backend();

// reset login state if user quit the app during login
if((['connecting', 'failed']: Array<LoginState>).includes(store.getState().user.status)) {
  store.dispatch(userActions.loginChange({
    status: 'none'
  }));
}

// reset connection state if user quit the app when connecting
if(store.getState().connect.status === 'connecting') {
  store.dispatch(connectActions.connectionChange({
    status: 'disconnected'
  }));
}

// Tray icon

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
  const { connect } = store.getState();
  ipcRenderer.send('changeTrayIcon', getIconType(connect.status));
};

// Setup primary event handlers to translate backend events into redux dispatch
mapBackendEventsToReduxActions(backend, store);

// Setup routing based on backend events
mapBackendEventsToRouter(backend, store);

ipcRenderer.on('backend-info', (_event, args) => {
  backend.setLocation(args.addr);
  backend.sync();
});
// Setup events to update tray icon
backend.on('connect', updateTrayIcon);
backend.on('connecting', updateTrayIcon);
backend.on('disconnect', updateTrayIcon);

// force update tray
updateTrayIcon();

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

const containerId = document.currentScript.getAttribute('data-container');
if(!containerId) {
  throw new Error('Missing data-container attribute.');
}

const rootElement = document.querySelector(containerId);
if(!rootElement) {
  throw new Error('Missing root element.');
}

ReactDOM.render(
  <Provider store={ store }>
    <ConnectedRouter history={ memoryHistory }>
    { makeRoutes(store.getState, { backend }) }
    </ConnectedRouter>
  </Provider>,
  rootElement
);