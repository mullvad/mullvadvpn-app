// @flow
import { createAction } from 'redux-actions';

import type { Backend } from '../lib/backend';
import type { UserReduxState } from '../reducers/user';
import type { ReduxAction } from '../store';

export type LoginChangeAction = <T: $Shape<UserReduxState>>(state: T) => ReduxAction<T>;

const loginChange: LoginChangeAction = createAction('USER_LOGIN_CHANGE');
const login = (backend: Backend, account: string) => () => backend.login(account);
const logout = (backend: Backend) => () => backend.logout();

export default { login, logout, loginChange };
