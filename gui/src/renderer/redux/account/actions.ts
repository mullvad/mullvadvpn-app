import { hasExpired } from '../../../shared/account-expiry';
import { AccountDataError, AccountNumber, IDevice } from '../../../shared/daemon-rpc-types';

interface IStartLoginAction {
  type: 'START_LOGIN';
  accountNumber: AccountNumber;
}

interface ILoggedInAction {
  type: 'LOGGED_IN';
  accountNumber: AccountNumber;
  deviceName?: string;
}

interface ILoginFailedAction {
  type: 'LOGIN_FAILED';
  error: AccountDataError['error'];
}

interface ILoginTooManyDevicesAction {
  type: 'TOO_MANY_DEVICES';
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
  accountNumber: AccountNumber;
  deviceName?: string;
  expiry: string;
}

interface IAccountSetupFinished {
  type: 'ACCOUNT_SETUP_FINISHED';
}

interface IHideNewDeviceBanner {
  type: 'HIDE_NEW_DEVICE_BANNER';
}

interface IUpdateAccountNumberAction {
  type: 'UPDATE_ACCOUNT_NUMBER';
  accountNumber: AccountNumber;
}

interface IUpdateAccountHistoryAction {
  type: 'UPDATE_ACCOUNT_HISTORY';
  accountHistory?: AccountNumber;
}

interface IUpdateAccountExpiryAction {
  type: 'UPDATE_ACCOUNT_EXPIRY';
  expiry?: string;
  expired: boolean;
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
  | ILoggedOutAction
  | IResetLoginErrorAction
  | IDeviceRevokedAction
  | IStartCreateAccount
  | ICreateAccountFailed
  | IAccountCreated
  | IAccountSetupFinished
  | IHideNewDeviceBanner
  | IUpdateAccountNumberAction
  | IUpdateAccountHistoryAction
  | IUpdateAccountExpiryAction
  | IUpdateDevicesAction;

function startLogin(accountNumber: AccountNumber): IStartLoginAction {
  return {
    type: 'START_LOGIN',
    accountNumber,
  };
}

function loggedIn(accountNumber: AccountNumber, device?: IDevice): ILoggedInAction {
  return {
    type: 'LOGGED_IN',
    accountNumber,
    deviceName: device?.name,
  };
}

function loginFailed(error: AccountDataError['error']): ILoginFailedAction {
  return {
    type: 'LOGIN_FAILED',
    error,
  };
}

function loginTooManyDevices(): ILoginTooManyDevicesAction {
  return {
    type: 'TOO_MANY_DEVICES',
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
  accountNumber: AccountNumber,
  device: IDevice | undefined,
  expiry: string,
): IAccountCreated {
  return {
    type: 'ACCOUNT_CREATED',
    accountNumber: accountNumber,
    deviceName: device?.name,
    expiry,
  };
}

function accountSetupFinished(): IAccountSetupFinished {
  return { type: 'ACCOUNT_SETUP_FINISHED' };
}

function hideNewDeviceBanner(): IHideNewDeviceBanner {
  return { type: 'HIDE_NEW_DEVICE_BANNER' };
}

function updateAccountNumber(accountNumber: AccountNumber): IUpdateAccountNumberAction {
  return {
    type: 'UPDATE_ACCOUNT_NUMBER',
    accountNumber,
  };
}

function updateAccountHistory(accountHistory?: AccountNumber): IUpdateAccountHistoryAction {
  return {
    type: 'UPDATE_ACCOUNT_HISTORY',
    accountHistory,
  };
}

function updateAccountExpiry(expiry?: string): IUpdateAccountExpiryAction {
  return {
    type: 'UPDATE_ACCOUNT_EXPIRY',
    expiry,
    expired: expiry !== undefined && hasExpired(expiry),
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
  loggedOut,
  resetLoginError,
  deviceRevoked,
  startCreateAccount,
  createAccountFailed,
  accountCreated,
  accountSetupFinished,
  hideNewDeviceBanner,
  updateAccountNumber,
  updateAccountHistory,
  updateAccountExpiry,
  updateDevices,
};
