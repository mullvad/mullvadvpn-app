// @flow
import type {
  DaemonRpcProtocol,
  AccountToken,
  AccountData,
  BackendState,
} from '../../app/lib/daemon-rpc';

interface MockRpc {
  sendNewState: (BackendState) => void;
  -getAccountData: (AccountToken) => Promise<AccountData>;
  -connectTunnel: () => Promise<void>;
  -getAccount: () => Promise<?AccountToken>;
  -authenticate: (string) => Promise<void>;
}

export function newMockRpc() {
  const stateListeners = [];
  const openListeners = [];
  const closeListeners = [];

  const mockIpc: DaemonRpcProtocol & MockRpc = {
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
      for (const listener of openListeners) {
        listener();
      }
    },
    disconnect: () => {
      for (const listener of closeListeners) {
        listener();
      }
    },
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
      openListeners.push(listener);
      return {
        unsubscribe: () => {},
      };
    },
    addCloseConnectionObserver: (listener: (error: ?Error) => void) => {
      closeListeners.push(listener);
      return {
        unsubscribe: () => {},
      };
    },
    authenticate: (_secret: string) => Promise.resolve(),
    getAccountHistory: () => Promise.resolve([]),
    removeAccountFromHistory: (_accountToken) => Promise.resolve(),
  };

  return mockIpc;
}
