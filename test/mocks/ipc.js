// @flow
import type { IpcFacade, BackendState } from '../../app/lib/ipc-facade';

interface MockIpc {
  sendNewState: (BackendState) => void;
  -getAccountData: *;
  -connect: *;
}

export function newMockIpc() {

  const stateListeners = [];

  const mockIpc: IpcFacade & MockIpc = {

    getAccountData: () => {
      return new Promise(r => r({
        paid_until: '',
      }));
    },
    setAccount: () => {
      return new Promise(r => r());
    },
    setCountry: () => {
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
        latlong: [],
      }));
    },
    getState: () => {
      return new Promise(r => r('unsecured'));
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
