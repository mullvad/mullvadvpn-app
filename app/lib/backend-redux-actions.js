import log from 'electron-log';
import userActions from '../actions/user';
import connectActions from '../actions/connect';

/**
 * Add event listeners to translate backend events to redux dispatch.
 *
 * @export
 * @param {Backend} backend
 * @param {Redux.Store} store
 */
export default function mapBackendEventsToReduxActions(backend, store) {
  const onUpdateIp = (clientIp) => {
    store.dispatch(connectActions.connectionChange({ clientIp }));
  };

  const onUpdateLocation = (data) => {
    store.dispatch(userActions.loginChange(data));
  };

  const onConnecting = (serverAddress) => {
    store.dispatch(connectActions.connectionChange({
      status: 'connecting',
      serverAddress
    }));
  };

  const onConnect = (serverAddress, error) => {
    if (error) {
      log.error('Unable to connect to', serverAddress, error);
    } else {
      store.dispatch(connectActions.connectionChange({ status: 'connected' }));
    }
  };

  const onDisconnect = () => {
    store.dispatch(connectActions.connectionChange({
      status: 'disconnected',
      serverAddress: null
    }));
  };

  const onLoggingIn = (info) => {
    store.dispatch(userActions.loginChange(Object.assign({
      status: 'connecting',
      error: null
    }, info)));
  };

  const onLogin = (info, error) => {
    const status = error ? 'failed' : 'ok';
    const paidUntil = info.paidUntil ? info.paidUntil : null;
    store.dispatch(userActions.loginChange({ paidUntil, status, error }));
  };

  const onLogout = () => {
    store.dispatch(userActions.loginChange({
      status: 'none',
      account: '',
      paidUntil: null,
      error: null
    }));
  };

  const onReachability = (isOnline) => {
    store.dispatch(connectActions.connectionChange({ isOnline }));
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
