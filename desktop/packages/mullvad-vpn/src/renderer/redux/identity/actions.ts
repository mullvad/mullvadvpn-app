import { IWireguardIdentity } from '../../../shared/daemon-rpc-types';

export interface IUpdateWireguardIdentityAction {
  type: 'UPDATE_WIREGUARD_IDENTITY';
  identity?: IWireguardIdentity;
}

export type IdentityAction = IUpdateWireguardIdentityAction;

function updateWireguardIdentity(identity?: IWireguardIdentity): IUpdateWireguardIdentityAction {
  return {
    type: 'UPDATE_WIREGUARD_IDENTITY',
    identity,
  };
}

export default {
  updateWireguardIdentity,
};
