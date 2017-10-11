// @flow
import type { IpcFacade, BackendState } from '../../app/lib/ipc-facade';

interface MockIpc {
  sendNewState: (BackendState) => void;
  -getAccountData: *;
  -connect: *;
  -getAccount: *;
}

export function newMockIpc() {

  const stateListeners = [];

  const mockIpc: IpcFacade & MockIpc = {

    setConnectionString: (_str: string) => {},
    authenticate: (_sharedSecret) => {
      return new Promise(r => r());
    },
    getAccountData: (accountToken) => {
      return new Promise(r => r({
        accountToken: accountToken,
        expiry: '',
      }));
    },
    getAccount: () => {
      return new Promise(r => r('1111'));
    },
    setAccount: () => {
      return new Promise(r => r());
    },
    setCustomRelay: () => {
      return new Promise(r => r());
    },
    connect: () => {
      return new Promise(r => r());
    },
    disconnect: () => {
      return new Promise(r => r());
    },
    getIp: () => {
      return new Promise(r => r('1.2.3.4'));
    },
    getLocation: () => {
      return new Promise(r => r({
        city: '',
        country: '',
        latlong: [0, 0],
      }));
    },
    getState: () => {
      return new Promise(r => r({
        state: 'unsecured',
        target_state:'unsecured',
      }));
    },
    registerStateListener: (listener: (BackendState) => void) => {
      stateListeners.push(listener);
    },
    sendNewState: (state: BackendState) => {
      for(const l of stateListeners) {
        l(state);
      }
    },
  };

  return mockIpc;
}
