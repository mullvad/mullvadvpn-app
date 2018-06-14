// @flow
import type { IpcFacade, AccountToken, AccountData, BackendState } from '../../app/lib/ipc-facade';

interface MockIpc {
  sendNewState: (BackendState) => void;
  killWebSocket: () => void;
  -getAccountData: (AccountToken) => Promise<AccountData>;
  -connect: () => Promise<void>;
  -getAccount: () => Promise<?AccountToken>;
  -authenticate: (string) => Promise<void>;
}

export function newMockIpc() {
  const stateListeners = [];
  const connectionCloseListeners = [];

  const mockIpc: IpcFacade & MockIpc = {
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

    connect: () => Promise.resolve(),

    disconnect: () => Promise.resolve(),

    shutdown: () => Promise.resolve(),

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

    registerStateListener: (listener: (BackendState) => void) => {
      stateListeners.push(listener);
    },

    sendNewState: (state: BackendState) => {
      for (const listener of stateListeners) {
        listener(state);
      }
    },

    setCloseConnectionHandler: (listener: () => void) => {
      connectionCloseListeners.push(listener);
    },

    authenticate: (_secret: string) => Promise.resolve(),

    getAccountHistory: () => Promise.resolve([]),

    removeAccountFromHistory: (_accountToken) => Promise.resolve(),

    killWebSocket: () => {
      for (const listener of connectionCloseListeners) {
        listener();
      }
    },
  };

  return mockIpc;
}
