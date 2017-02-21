import React from 'react';
import ReactDOM from 'react-dom';
import { Provider } from 'react-redux';
import { Router, hashHistory } from 'react-router';
import { syncHistoryWithStore } from 'react-router-redux';
import { webFrame } from 'electron';
import routes from './routes';
import configureStore from './store';
import userActions from './actions/user';
import connectActions from './actions/connect';
import Backend from './lib/backend';
import { LoginState, ConnectionState } from './constants';

const initialState = {};
const store = configureStore(initialState);

// desperately trying to fix https://github.com/reactjs/react-router-redux/issues/534
hashHistory.replace('/');

// see https://github.com/reactjs/react-router-redux/issues/534
const recentLocation = (store.getState().routing || {}).locationBeforeTransitions;
const routerHistory = syncHistoryWithStore(hashHistory, store, { adjustUrlOnReplay: true });

if(recentLocation && recentLocation.pathname) {
  routerHistory.replace(recentLocation.pathname);
}

const rootElement = document.querySelector(document.currentScript.getAttribute('data-container'));

// disable smart pinch.
webFrame.setVisualZoomLevelLimits(1, 1);

// Create backend
const backend = new Backend();

// Setup events

backend.on(Backend.EventType.connecting, (serverAddress) => {
  store.dispatch(connectActions.connectionChange({ 
    status: ConnectionState.connecting,
    error: null,
    serverAddress
  }));
});

backend.on(Backend.EventType.connect, (serverAddress, error) => {
  const status = error ? ConnectionState.disconnected : ConnectionState.connected;
  store.dispatch(connectActions.connectionChange({ error, status }));
});

backend.on(Backend.EventType.disconnect, () => {
  store.dispatch(connectActions.connectionChange({
    status: ConnectionState.disconnected,
    serverAddress: null, 
    error: null
  }));
});

backend.on(Backend.EventType.logging, (account) => {
  store.dispatch(userActions.loginChange({ 
    status: LoginState.connecting, 
    error: null,
    account
  }));
});

backend.on(Backend.EventType.login, (account, error) => {
  const status = error ? LoginState.failed : LoginState.ok;
  store.dispatch(userActions.loginChange({ status, error }));
});

backend.on(Backend.EventType.logout, () => {
  store.dispatch(userActions.loginChange({
    status: LoginState.none, 
    account: null,
    error: null
  }));
});

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
