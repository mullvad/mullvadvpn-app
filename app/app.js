import React from 'react';
import ReactDOM from 'react-dom';
import { Provider } from 'react-redux';
import { Router, createMemoryHistory } from 'react-router';
import { syncHistoryWithStore, replace } from 'react-router-redux';
import { webFrame, ipcRenderer } from 'electron';
import makeRoutes from './routes';
import configureStore from './store';
import userActions from './actions/user';
import connectActions from './actions/connect';
import Backend from './lib/backend';
import mapBackendEventsToReduxActions from './lib/backend-redux-actions';
import { LoginState, ConnectionState } from './constants';

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
if([ConnectionState.connecting, ConnectionState.failed].includes(store.getState().connect.status)) {
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

const rootElement = document.querySelector(document.currentScript.getAttribute('data-container'));

// disable smart pinch.
webFrame.setVisualZoomLevelLimits(1, 1);

// Tray icon
const updateTrayIcon = () => {
  const getName = (s) => {
    switch(s) {
    case ConnectionState.connected: return 'connected';
    default: return 'default';
    }
  };
  const { connect } = store.getState();
  ipcRenderer.send('changeTrayIcon', getName(connect.status));
};

/**
 * Patch backend state.
 * 
 * Currently backend does not have external state
 * such as VPN connection status or IP address.
 * 
 * So far we store everything in redux and have to
 * sync redux state with backend.
 * 
 * In future this will be the other way around.
 */
const syncBackendWithReduxStore = (backend, store) => {
  const mapConnStatus = (s) => {
    const S = ConnectionState;
    const BS = Backend.ConnectionState;
    switch(s) {
    case S.connected: return BS.connected;
    case S.connecting: return BS.connecting;
    default: return BS.disconnected;
    }
  };
  const { connect } = store.getState();
  backend._connStatus = mapConnStatus(connect.status);
};
syncBackendWithReduxStore(backend, store);

// Setup primary event handlers to translate backend events into redux dispatch
mapBackendEventsToReduxActions(backend, store);

// redirect user to main screen after login
backend.on(Backend.EventType.login, (addr, error) => {
  if(error) { return; }
  setTimeout(() => store.dispatch(replace('/connect')), 1000);
});

// redirect user to login page on logout
backend.on(Backend.EventType.logout, () => store.dispatch(replace('/')));

// Setup events to update tray icon
backend.on(Backend.EventType.connect, updateTrayIcon);
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

ReactDOM.render(
  <Provider store={ store }>
    <Router history={ routerHistory } routes={ routes } createElement={ createElement } />
  </Provider>,
  rootElement
);
