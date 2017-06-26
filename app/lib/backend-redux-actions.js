// @flow

import log from 'electron-log';
import accountActions from '../redux/account/actions.js';
import connectionActions from '../redux/connection/actions.js';
import { Backend } from './backend.js';
import type { ReduxStore } from '../redux/store.js';

/**
 * Add event listeners to translate backend events to redux dispatch.
 *
 * @export
 * @param {Backend} backend
 * @param {Redux.Store} store
 */
export default function mapBackendEventsToReduxActions(backend: Backend, store: ReduxStore) {

  const onUpdateIp = (clientIp) => {
    store.dispatch(connectionActions.connectionChange({ clientIp }));
  };

  const onUpdateLocation = (data) => {
    store.dispatch(accountActions.loginChange(data));
  };

  const onConnecting = (serverAddress) => {
    store.dispatch(connectionActions.connectionChange({
      status: 'connecting',
      serverAddress
    }));
  };

  const onConnect = (serverAddress, error) => {
    if (error) {
      log.error('Unable to connect to', serverAddress, error);
    } else {
      store.dispatch(connectionActions.connectionChange({ status: 'connected' }));
    }
  };

  const onDisconnect = () => {
    store.dispatch(connectionActions.connectionChange({
      status: 'disconnected',
      serverAddress: null
    }));
  };

  const onLoggingIn = (info) => {
    store.dispatch(accountActions.loginChange(Object.assign({
      status: 'connecting',
      error: null
    }, info)));
  };

  const onLogin = (info, error) => {
    const status = error ? 'failed' : 'ok';
    const paidUntil = info.paidUntil ? info.paidUntil : null;
    store.dispatch(accountActions.loginChange({ paidUntil, status, error }));
  };

  const onLogout = () => {
    store.dispatch(accountActions.loginChange({
      status: 'none',
      account: '',
      paidUntil: null,
      error: null
    }));
  };

  const onReachability = (isOnline) => {
    store.dispatch(connectionActions.connectionChange({ isOnline }));
  };

  backend.on('updatedIp', onUpdateIp);
  backend.on('updatedLocation', onUpdateLocation);
  backend.on('connecting', onConnecting);
  backend.on('connect', onConnect);
  backend.on('disconnect', onDisconnect);
  backend.on('logging', onLoggingIn);
  backend.on('login', onLogin);
  backend.on('logout', onLogout);
  backend.on('updatedReachability', onReachability);
}
