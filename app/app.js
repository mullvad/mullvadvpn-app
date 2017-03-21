import path from 'path';
import React from 'react';
import ReactDOM from 'react-dom';
import { Provider } from 'react-redux';
import { Router, createMemoryHistory } from 'react-router';
import { syncHistoryWithStore } from 'react-router-redux';
import { webFrame, ipcRenderer } from 'electron';
import makeRoutes from './routes';
import configureStore from './store';
import userActions from './actions/user';
import connectActions from './actions/connect';
import Backend from './lib/backend';
import mapBackendEventsToReduxActions from './lib/backend-redux-actions';
import mapBackendEventsToRouter from './lib/backend-routing';
import { LoginState, ConnectionState, TrayIconType } from './enums';

const initialState = {};
const memoryHistory = createMemoryHistory();
const store = configureStore(initialState, memoryHistory);
const routes = makeRoutes(store);
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

// desperately trying to fix https://github.com/reactjs/react-router-redux/issues/534
memoryHistory.replace('/');

const recentLocation = (store.getState().routing || {}).locationBeforeTransitions;
const routerHistory = syncHistoryWithStore(memoryHistory, store, { adjustUrlOnReplay: true });
if(recentLocation && recentLocation.pathname) {
  routerHistory.replace(recentLocation.pathname);
}

// Tray icon

/**
 * Get tray icon type based on connection state
 * @param    {ConnectionState} s - connection state
 * @return   {TrayIconType}
 * 
 */
const getIconType = (s) => {
  switch(s) {
  case ConnectionState.connected: return TrayIconType.secured;
  case ConnectionState.connecting: return TrayIconType.securing;
  default: return TrayIconType.unsecured;
  }
};

/**
 * Update tray icon via IPC call
 */
const updateTrayIcon = () => {
  const { connect } = store.getState();
  ipcRenderer.send('changeTrayIcon', getIconType(connect.status));
};

// patch backend
backend.syncWithReduxStore(store);

// Setup primary event handlers to translate backend events into redux dispatch
mapBackendEventsToReduxActions(backend, store);

// Setup routing based on backend events
mapBackendEventsToRouter(backend, store);

// Setup events to update tray icon
backend.on(Backend.EventType.connect, updateTrayIcon);
backend.on(Backend.EventType.connecting, updateTrayIcon);
backend.on(Backend.EventType.disconnect, updateTrayIcon);

// force update tray
updateTrayIcon();

// helper method for router to pass backend down the component tree
const createElement = (Component, props) => {
  const newProps = { ...props, backend };
  return (
    <Component {...newProps} />
  );
};

const rootElement = document.querySelector(document.currentScript.getAttribute('data-container'));

// disable smart pinch.
webFrame.setZoomLevelLimits(1, 1);

if ('serviceWorker' in navigator) {
  navigator.serviceWorker.register(path.join(__dirname, 'tilecache.sw.js'))
    .then((registration) => {
      console.log('ServiceWorker registration successful with scope: ', registration.scope);
    }).catch((err) => {
      console.log('ServiceWorker registration failed: ', err);
    });
}

ReactDOM.render(
  <Provider store={ store }>
    <Router history={ routerHistory } routes={ routes } createElement={ createElement } />
  </Provider>,
  rootElement
);
