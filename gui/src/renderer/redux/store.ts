import { combineReducers, createStore, Dispatch } from 'redux';

import { AccountAction } from './account/actions';
import accountReducer, { IAccountReduxState } from './account/reducers';
import { ConnectionAction } from './connection/actions';
import connectionReducer, { IConnectionReduxState } from './connection/reducers';
import { SettingsAction } from './settings/actions';
import settingsReducer, { ISettingsReduxState } from './settings/reducers';
import { SupportAction } from './support/actions';
import supportReducer, { ISupportReduxState } from './support/reducers';
import { UserInterfaceAction } from './userinterface/actions';
import userInterfaceReducer, { IUserInterfaceReduxState } from './userinterface/reducers';
import { VersionAction } from './version/actions';
import versionReducer, { IVersionReduxState } from './version/reducers';

export interface IReduxState {
  account: IAccountReduxState;
  connection: IConnectionReduxState;
  settings: ISettingsReduxState;
  support: ISupportReduxState;
  version: IVersionReduxState;
  userInterface: IUserInterfaceReduxState;
}

export type ReduxAction =
  | AccountAction
  | ConnectionAction
  | SettingsAction
  | SupportAction
  | VersionAction
  | UserInterfaceAction;
export type ReduxStore = ReturnType<typeof configureStore>;
export type ReduxDispatch = Dispatch<ReduxAction>;

export default function configureStore(initialState?: IReduxState) {
  const reducers = {
    account: accountReducer,
    connection: connectionReducer,
    settings: settingsReducer,
    support: supportReducer,
    version: versionReducer,
    userInterface: userInterfaceReducer,
  };

  const rootReducer = combineReducers(reducers);

  if (initialState) {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    return createStore(rootReducer, initialState, (window as any).__REDUX_DEVTOOLS_EXTENSION__?.());
  } else {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    return createStore(rootReducer, (window as any).__REDUX_DEVTOOLS_EXTENSION__?.());
  }
}
