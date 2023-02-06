import { AccountToken, IDevice } from '../../../shared/daemon-rpc-types';

interface IStartLoginAction {
  type: 'START_LOGIN';
  accountToken: AccountToken;
}

interface ILoggedInAction {
  type: 'LOGGED_IN';
  accountToken: AccountToken;
  deviceName?: string;
}

interface ILoginFailedAction {
  type: 'LOGIN_FAILED';
  error: Error;
}

interface ILoginTooManyDevicesAction {
  type: 'TOO_MANY_DEVICES';
  error: Error;
}

interface IPrepareLogoutAction {
  type: 'PREPARE_LOG_OUT';
}

interface ICancelLogoutAction {
  type: 'CANCEL_LOGOUT';
}

interface ILoggedOutAction {
  type: 'LOGGED_OUT';
}

interface IResetLoginErrorAction {
  type: 'RESET_LOGIN_ERROR';
}

interface IDeviceRevokedAction {
  type: 'DEVICE_REVOKED';
}

interface IStartCreateAccount {
  type: 'START_CREATE_ACCOUNT';
}

interface ICreateAccountFailed {
  type: 'CREATE_ACCOUNT_FAILED';
  error: Error;
}

interface IAccountCreated {
  type: 'ACCOUNT_CREATED';
  accountToken: AccountToken;
  deviceName?: string;
  expiry: string;
}

interface IAccountSetupFinished {
  type: 'ACCOUNT_SETUP_FINISHED';
}

interface IUpdateAccountTokenAction {
  type: 'UPDATE_ACCOUNT_TOKEN';
  accountToken: AccountToken;
}

interface IUpdateAccountHistoryAction {
  type: 'UPDATE_ACCOUNT_HISTORY';
  accountHistory?: AccountToken;
}

interface IUpdateAccountExpiryAction {
  type: 'UPDATE_ACCOUNT_EXPIRY';
  expiry?: string;
}

interface IUpdateDevicesAction {
  type: 'UPDATE_DEVICES';
  devices: Array<IDevice>;
}

export type AccountAction =
  | IStartLoginAction
  | ILoggedInAction
  | ILoginFailedAction
  | ILoginTooManyDevicesAction
  | IPrepareLogoutAction
  | ICancelLogoutAction
  | ILoggedOutAction
  | IResetLoginErrorAction
  | IDeviceRevokedAction
  | IStartCreateAccount
  | ICreateAccountFailed
  | IAccountCreated
  | IAccountSetupFinished
  | IUpdateAccountTokenAction
  | IUpdateAccountHistoryAction
  | IUpdateAccountExpiryAction
  | IUpdateDevicesAction;

function startLogin(accountToken: AccountToken): IStartLoginAction {
  return {
    type: 'START_LOGIN',
    accountToken,
  };
}

function loggedIn(accountToken: AccountToken, device?: IDevice): ILoggedInAction {
  return {
    type: 'LOGGED_IN',
    accountToken,
    deviceName: device?.name,
  };
}

function loginFailed(error: Error): ILoginFailedAction {
  return {
    type: 'LOGIN_FAILED',
    error,
  };
}

function loginTooManyDevices(error: Error): ILoginTooManyDevicesAction {
  return {
    type: 'TOO_MANY_DEVICES',
    error,
  };
}

function prepareLogout(): IPrepareLogoutAction {
  return {
    type: 'PREPARE_LOG_OUT',
  };
}

function cancelLogout(): ICancelLogoutAction {
  return {
    type: 'CANCEL_LOGOUT',
  };
}

function loggedOut(): ILoggedOutAction {
  return {
    type: 'LOGGED_OUT',
  };
}

function resetLoginError(): IResetLoginErrorAction {
  return {
    type: 'RESET_LOGIN_ERROR',
  };
}

function deviceRevoked(): IDeviceRevokedAction {
  return {
    type: 'DEVICE_REVOKED',
  };
}

function startCreateAccount(): IStartCreateAccount {
  return {
    type: 'START_CREATE_ACCOUNT',
  };
}

function createAccountFailed(error: Error): ICreateAccountFailed {
  return {
    type: 'CREATE_ACCOUNT_FAILED',
    error,
  };
}

function accountCreated(
  accountToken: AccountToken,
  device: IDevice | undefined,
  expiry: string,
): IAccountCreated {
  return {
    type: 'ACCOUNT_CREATED',
    accountToken: accountToken,
    deviceName: device?.name,
    expiry,
  };
}

function accountSetupFinished(): IAccountSetupFinished {
  return { type: 'ACCOUNT_SETUP_FINISHED' };
}

function updateAccountToken(accountToken: AccountToken): IUpdateAccountTokenAction {
  return {
    type: 'UPDATE_ACCOUNT_TOKEN',
    accountToken,
  };
}

function updateAccountHistory(accountHistory?: AccountToken): IUpdateAccountHistoryAction {
  return {
    type: 'UPDATE_ACCOUNT_HISTORY',
    accountHistory,
  };
}

function updateAccountExpiry(expiry?: string): IUpdateAccountExpiryAction {
  return {
    type: 'UPDATE_ACCOUNT_EXPIRY',
    expiry,
  };
}

function updateDevices(devices: Array<IDevice>): IUpdateDevicesAction {
  return {
    type: 'UPDATE_DEVICES',
    devices: devices.sort((a, b) => a.created.getTime() - b.created.getTime()),
  };
}

export default {
  startLogin,
  loggedIn,
  loginFailed,
  loginTooManyDevices,
  prepareLogout,
  cancelLogout,
  loggedOut,
  resetLoginError,
  deviceRevoked,
  startCreateAccount,
  createAccountFailed,
  accountCreated,
  accountSetupFinished,
  updateAccountToken,
  updateAccountHistory,
  updateAccountExpiry,
  updateDevices,
};
