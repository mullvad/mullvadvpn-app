import { AccountDataError, AccountToken, IDevice } from '../../../shared/daemon-rpc-types';
import { ReduxAction } from '../store';

type LoginMethod = 'existing_account' | 'new_account';
type ExpiredState = 'expired' | 'time_added';

export type LoginState =
  | { type: 'none'; deviceRevoked: boolean }
  | { type: 'logging in'; method: LoginMethod }
  | { type: 'ok'; method: LoginMethod; newDeviceBanner: boolean; expiredState?: ExpiredState }
  | { type: 'too many devices'; method: LoginMethod }
  | { type: 'failed'; method: 'existing_account'; error: AccountDataError['error'] }
  | { type: 'failed'; method: 'new_account'; error: Error };
export interface IAccountReduxState {
  accountToken?: AccountToken;
  deviceName?: string;
  devices: Array<IDevice>;
  accountHistory?: AccountToken;
  expiry?: string; // ISO8601
  status: LoginState;
}

const initialState: IAccountReduxState = {
  accountToken: undefined,
  deviceName: undefined,
  devices: [],
  accountHistory: undefined,
  expiry: undefined,
  status: { type: 'none', deviceRevoked: false },
};

export default function (
  state: IAccountReduxState = initialState,
  action: ReduxAction,
): IAccountReduxState {
  switch (action.type) {
    case 'START_LOGIN':
      return {
        ...state,
        status: { type: 'logging in', method: 'existing_account' },
        accountToken: action.accountToken,
      };
    case 'LOGGED_IN':
      return {
        ...state,
        status: {
          type: 'ok',
          method: 'existing_account',
          newDeviceBanner: state.status.type === 'logging in',
        },
        accountToken: action.accountToken,
        deviceName: action.deviceName,
      };
    case 'LOGIN_FAILED':
      return {
        ...state,
        status: { type: 'failed', method: 'existing_account', error: action.error },
      };
    case 'TOO_MANY_DEVICES':
      return {
        ...state,
        status: { type: 'too many devices', method: 'existing_account' },
      };
    case 'LOGGED_OUT':
      return {
        ...state,
        status: { type: 'none', deviceRevoked: false },
        accountToken: undefined,
        expiry: undefined,
      };
    case 'RESET_LOGIN_ERROR':
      return {
        ...state,
        status: { type: 'none', deviceRevoked: false },
      };
    case 'DEVICE_REVOKED':
      return {
        ...state,
        status: { type: 'none', deviceRevoked: true },
      };
    case 'START_CREATE_ACCOUNT':
      return {
        ...state,
        status: { type: 'logging in', method: 'new_account' },
      };
    case 'CREATE_ACCOUNT_FAILED':
      return {
        ...state,
        status: { type: 'failed', method: 'new_account', error: action.error },
      };
    case 'ACCOUNT_CREATED':
      return {
        ...state,
        status: {
          type: 'ok',
          method: 'new_account',
          newDeviceBanner: true,
          expiredState: 'expired',
        },
        accountToken: action.accountToken,
        deviceName: action.deviceName,
        expiry: action.expiry,
      };
    case 'ACCOUNT_SETUP_FINISHED':
      return {
        ...state,
        status: { type: 'ok', method: 'existing_account', newDeviceBanner: true },
      };
    case 'HIDE_NEW_DEVICE_BANNER':
      if (state.status.type !== 'ok') {
        return state;
      }

      return {
        ...state,
        status: { ...state.status, newDeviceBanner: false },
      };
    case 'UPDATE_ACCOUNT_TOKEN':
      return {
        ...state,
        accountToken: action.accountToken,
      };
    case 'UPDATE_ACCOUNT_HISTORY':
      return {
        ...state,
        accountHistory: action.accountHistory,
      };
    case 'UPDATE_ACCOUNT_EXPIRY': {
      const status = { ...state.status };
      if (status.type === 'ok') {
        if (action.expired) {
          status.expiredState = 'expired';
        } else if (status.expiredState === 'expired' && !action.expired) {
          status.expiredState = 'time_added';
        } else {
          status.expiredState = undefined;
        }
      }

      return {
        ...state,
        expiry: action.expiry,
        status,
      };
    }
    case 'UPDATE_DEVICES':
      return {
        ...state,
        devices: action.devices,
      };
  }

  return state;
}
