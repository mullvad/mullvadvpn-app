import { AccountToken, IDevice } from '../../../shared/daemon-rpc-types';
import { ReduxAction } from '../store';

type LoginMethod = 'existing_account' | 'new_account';
export type LoginState =
  | { type: 'none' }
  | { type: 'logging in' | 'ok'; method: LoginMethod }
  | { type: 'failed' | 'too many devices'; method: LoginMethod; error: Error };
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
  status: { type: 'none' },
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
        status: { type: 'ok', method: 'existing_account' },
        accountToken: action.accountToken,
        deviceName: action.deviceName,
      };
    case 'LOGIN_FAILED':
      return {
        ...state,
        status: { type: 'failed', method: 'existing_account', error: action.error },
        accountToken: undefined,
      };
    case 'TOO_MANY_DEVICES':
      return {
        ...state,
        status: { type: 'too many devices', method: 'existing_account', error: action.error },
      };
    case 'LOGGED_OUT':
      return {
        ...state,
        status: { type: 'none' },
        accountToken: undefined,
        expiry: undefined,
      };
    case 'RESET_LOGIN_ERROR':
      return {
        ...state,
        status: { type: 'none' },
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
        status: { type: 'ok', method: 'new_account' },
        accountToken: action.accountToken,
        deviceName: action.deviceName,
        expiry: action.expiry,
      };
    case 'ACCOUNT_SETUP_FINISHED':
      return {
        ...state,
        status: { type: 'ok', method: 'existing_account' },
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
    case 'UPDATE_ACCOUNT_EXPIRY':
      return {
        ...state,
        expiry: action.expiry,
      };
    case 'UPDATE_DEVICES':
      return {
        ...state,
        devices: action.devices,
      };
  }

  return state;
}
