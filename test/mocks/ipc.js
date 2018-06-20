// @flow
import type {
  DaemonRpcProtocol,
  AccountToken,
  AccountData,
  BackendState,
} from '../../app/lib/daemon-rpc';

interface MockIpc {
  sendNewState: (BackendState) => void;
  killWebSocket: () => void;
  -getAccountData: (AccountToken) => Promise<AccountData>;
  -connectTunnel: () => Promise<void>;
  -getAccount: () => Promise<?AccountToken>;
  -authenticate: (string) => Promise<void>;
}

export function newMockIpc() {
  const stateListeners = [];
  let connectionOpenListener: ?() => void;
  let connectionCloseListener: ?(error: ?Error) => void;

  const mockIpc: DaemonRpcProtocol & MockIpc = {
    setConnectionString: (_str: string) => {},
    getAccountData: (accountToken) =>
      Promise.resolve({
        accountToken: accountToken,
        expiry: '',
      }),
    getRelayLocations: () =>
      Promise.resolve({
        countries: [],
      }),
    getAccount: () => Promise.resolve('1111'),
    setAccount: () => Promise.resolve(),
    updateRelaySettings: () => Promise.resolve(),
    getRelaySettings: () =>
      Promise.resolve({
        custom_tunnel_endpoint: {
          host: 'www.example.com',
          tunnel: {
            openvpn: {
              port: 1301,
              protocol: 'udp',
            },
          },
        },
      }),
    setAllowLan: (_allowLan: boolean) => Promise.resolve(),
    getAllowLan: () => Promise.resolve(true),
    connect: () => {
      if (connectionOpenListener) {
        connectionOpenListener();
      }
    },
    disconnect: () => {},
    connectTunnel: () => Promise.resolve(),
    disconnectTunnel: () => Promise.resolve(),
    getLocation: () =>
      Promise.resolve({
        ip: '',
        country: '',
        city: '',
        latitude: 0.0,
        longitude: 0.0,
        mullvad_exit_ip: false,
      }),
    getState: () =>
      Promise.resolve({
        state: 'unsecured',
        target_state: 'unsecured',
      }),
    subscribeStateListener: (listener: (state: ?BackendState, error: ?Error) => void) => {
      stateListeners.push(listener);
      return Promise.resolve();
    },
    sendNewState: (state: BackendState) => {
      for (const listener of stateListeners) {
        listener(state);
      }
    },
    addOpenConnectionObserver: (listener: () => void) => {
      connectionOpenListener = listener;
    },
    addCloseConnectionObserver: (listener: (error: ?Error) => void) => {
      connectionCloseListener = listener;
    },
    authenticate: (_secret: string) => Promise.resolve(),
    getAccountHistory: () => Promise.resolve([]),
    removeAccountFromHistory: (_accountToken) => Promise.resolve(),

    killWebSocket: () => {
      if (connectionCloseListener) {
        connectionCloseListener();
      }
    },
  };

  return mockIpc;
}
