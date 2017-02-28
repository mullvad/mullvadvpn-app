import { replace } from 'react-router-redux';
import userActions from '../actions/user';
import connectActions from '../actions/connect';
import Backend from './backend';
import { LoginState, ConnectionState } from '../constants';

export default function mapBackendEventsToReduxActions(backend, store) {
  const onUpdateIp = (clientIp) => {
    store.dispatch(connectActions.connectionChange({ clientIp }));
  };

  const onConnecting = (serverAddress) => {
    store.dispatch(connectActions.connectionChange({ 
      status: ConnectionState.connecting,
      error: null,
      serverAddress
    }));
  };

  const onConnect = (serverAddress, error) => {
    const status = error ? ConnectionState.disconnected : ConnectionState.connected;
    store.dispatch(connectActions.connectionChange({ error, status }));
  };

  const onDisconnect = () => {
    store.dispatch(connectActions.connectionChange({
      status: ConnectionState.disconnected,
      serverAddress: null, 
      error: null
    }));
  };

  const onLoggingIn = (account) => {
    store.dispatch(userActions.loginChange({ 
      status: LoginState.connecting, 
      error: null,
      account
    }));
  };

  const onLogin = (account, error) => {
    const status = error ? LoginState.failed : LoginState.ok;
    store.dispatch(userActions.loginChange({ status, error }));
    
    // redirect to main screen after delay
    if(status === LoginState.ok) {
      const preferredServer = store.getState().settings.preferredServer;
      const server = backend.serverInfo(preferredServer);

      // auto-connect
      setTimeout(() => backend.connect(server.address), 1000);
    }
  };

  const onLogout = () => {
    store.dispatch(userActions.loginChange({
      status: LoginState.none, 
      account: null,
      error: null
    }));
  };

  backend.on(Backend.EventType.updatedIp, onUpdateIp);
  backend.on(Backend.EventType.connecting, onConnecting);
  backend.on(Backend.EventType.connect, onConnect);
  backend.on(Backend.EventType.disconnect, onDisconnect);
  backend.on(Backend.EventType.logging, onLoggingIn);
  backend.on(Backend.EventType.login, onLogin);
  backend.on(Backend.EventType.logout, onLogout);
};
