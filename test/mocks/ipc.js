// @flow
import type { IpcFacade } from '../../app/lib/ipc-facade';

export function newMockIpc() {
  return Object.assign({}, mockIpc);
}

const mockIpc: IpcFacade = {

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
};
