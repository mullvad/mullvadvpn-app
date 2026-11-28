import { IWireguardIdentity } from '../../../shared/daemon-rpc-types';
import { ReduxAction } from '../store';

export interface IIdentityReduxState {
  // Not set as long as no WireGuard config has been imported.
  identity?: IWireguardIdentity;
}

const initialState: IIdentityReduxState = {
  identity: undefined,
};

export default function (
  state: IIdentityReduxState = initialState,
  action: ReduxAction,
): IIdentityReduxState {
  switch (action.type) {
    case 'UPDATE_WIREGUARD_IDENTITY':
      return {
        ...state,
        identity: action.identity,
      };
  }

  return state;
}
