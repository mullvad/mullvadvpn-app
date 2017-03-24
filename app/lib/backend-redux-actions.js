import userActions from '../actions/user';
import connectActions from '../actions/connect';
import Backend from './backend';
import { LoginState, ConnectionState } from '../enums';

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
      status: ConnectionState.connecting,
      serverAddress
    }));
  };

  const onConnect = (serverAddress) => {
    store.dispatch(connectActions.connectionChange({ status: ConnectionState.connected }));
  };

  const onDisconnect = () => {
    store.dispatch(connectActions.connectionChange({
      status: ConnectionState.disconnected,
      serverAddress: null
    }));
  };

  const onLoggingIn = (info) => {
    store.dispatch(userActions.loginChange(Object.assign({
      status: LoginState.connecting,
      error: null
    }, info)));
  };

  const onLogin = (info, error) => {
    const status = error ? LoginState.failed : LoginState.ok;
    const paidUntil = info.paidUntil ? info.paidUntil : null;
    store.dispatch(userActions.loginChange({ paidUntil, status, error }));
  };

  const onLogout = () => {
    store.dispatch(userActions.loginChange({
      status: LoginState.none,
      account: null,
      paidUntil: null,
      error: null
    }));
  };

  const onReachability = (isOnline) => {
    store.dispatch(connectActions.connectionChange({ isOnline }));
  };

  backend.on(Backend.EventType.updatedIp, onUpdateIp);
  backend.on(Backend.EventType.updatedLocation, onUpdateLocation);
  backend.on(Backend.EventType.connecting, onConnecting);
  backend.on(Backend.EventType.connect, onConnect);
  backend.on(Backend.EventType.disconnect, onDisconnect);
  backend.on(Backend.EventType.logging, onLoggingIn);
  backend.on(Backend.EventType.login, onLogin);
  backend.on(Backend.EventType.logout, onLogout);
  backend.on(Backend.EventType.updatedReachability, onReachability);
}
