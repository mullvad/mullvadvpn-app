// @flow

import type { Backend } from '../../lib/backend';
import type { AccountReduxState } from './reducers.js';

export type LoginChangeAction = {
  type:'LOGIN_CHANGE',
  newData: $Shape<AccountReduxState>,
};

function loginChange(data: $Shape<AccountReduxState>): LoginChangeAction {
  return {
    type: 'LOGIN_CHANGE',
    newData: data,
  };
}

const login = (backend: Backend, account: string) => () => backend.login(account);
const logout = (backend: Backend) => () => backend.logout();

export default { login, logout, loginChange };
