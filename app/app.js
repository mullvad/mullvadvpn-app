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
import Backend from './lib/backend';
import mapBackendEventsToReduxActions from './lib/backend-redux-actions';
import mapBackendEventsToRouter from './lib/backend-routing';
import { LoginState, ConnectionState } from './enums';
import type { TrayIconType } from './lib/tray-icon-manager';

const initialState = {};
const memoryHistory = createMemoryHistory();
const store = configureStore(initialState, memoryHistory);
const backend = new Backend();

// reset login state if user quit the app during login
if([LoginState.connecting, LoginState.failed].includes(store.getState().user.status)) {
  store.dispatch(userActions.loginChange({
    status: LoginState.none
  }));
}

// reset connection state if user quit the app when connecting
if(store.getState().connect.status === ConnectionState.connecting) {
  store.dispatch(connectActions.connectionChange({
    status: ConnectionState.disconnected
  }));
}

// Tray icon

/**
 * Get tray icon type based on connection state
 */
const getIconType = (s: string): TrayIconType => {
  switch(s) {
  case ConnectionState.connected: return 'secured';
  case ConnectionState.connecting: return 'securing';
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

ipcRenderer.on('backend-info', (event, args) => {
  backend.setLocation(args.addr);
  backend.sync();
});
// Setup events to update tray icon
backend.on(Backend.EventType.connect, updateTrayIcon);
backend.on(Backend.EventType.connecting, updateTrayIcon);
backend.on(Backend.EventType.disconnect, updateTrayIcon);

// force update tray
updateTrayIcon();

const rootElement = document.querySelector(document.currentScript.getAttribute('data-container'));

// disable smart pinch.
webFrame.setZoomLevelLimits(1, 1);

if ('serviceWorker' in navigator) {
  navigator.serviceWorker.register(path.join(__dirname, 'tilecache.sw.js'))
    .then((registration) => {
      log.info('ServiceWorker registration successful with scope: ', registration.scope);
    }).catch((err) => {
      log.info('ServiceWorker registration failed: ', err);
    });
}

ipcRenderer.send('on-browser-window-ready');

ReactDOM.render(
  <Provider store={ store }>
    <ConnectedRouter history={ memoryHistory }>
    { makeRoutes(store.getState, { backend }) }
    </ConnectedRouter>
  </Provider>,
  rootElement
);
