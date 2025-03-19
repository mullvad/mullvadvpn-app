import { useRef } from 'react';
import { useSelector as useReduxSelector } from 'react-redux';
import { combineReducers, compose, createStore, Dispatch, StoreEnhancer } from 'redux';

import { useWillExit } from '../lib/will-exit';
import accountActions, { AccountAction } from './account/actions';
import accountReducer, { IAccountReduxState } from './account/reducers';
import connectionActions, { ConnectionAction } from './connection/actions';
import connectionReducer, { IConnectionReduxState } from './connection/reducers';
import { downloadUpdateActions } from './download-update/actions';
import { downloadUpdateReducer, DownloadUpdateState } from './download-update/reducers';
import settingsActions, { SettingsAction } from './settings/actions';
import settingsReducer, { ISettingsReduxState } from './settings/reducers';
import { SettingsImportAction } from './settings-import/actions';
import settingsImportReducer, { SettingsImportReduxState } from './settings-import/reducers';
import supportActions, { SupportAction } from './support/actions';
import supportReducer, { ISupportReduxState } from './support/reducers';
import userInterfaceActions, { UserInterfaceAction } from './userinterface/actions';
import userInterfaceReducer, { IUserInterfaceReduxState } from './userinterface/reducers';
import versionActions, { VersionAction } from './version/actions';
import versionReducer, { IVersionReduxState } from './version/reducers';

export interface IReduxState {
  account: IAccountReduxState;
  connection: IConnectionReduxState;
  downloadUpdate: DownloadUpdateState;
  settings: ISettingsReduxState;
  support: ISupportReduxState;
  version: IVersionReduxState;
  userInterface: IUserInterfaceReduxState;
  settingsImport: SettingsImportReduxState;
}

export type ReduxAction =
  | AccountAction
  | ConnectionAction
  | SettingsAction
  | SupportAction
  | VersionAction
  | UserInterfaceAction
  | SettingsImportAction;
export type ReduxStore = ReturnType<typeof configureStore>;
export type ReduxDispatch = Dispatch<ReduxAction>;

export default function configureStore() {
  const reducers = {
    account: accountReducer,
    connection: connectionReducer,
    downloadUpdate: downloadUpdateReducer,
    settings: settingsReducer,
    support: supportReducer,
    version: versionReducer,
    userInterface: userInterfaceReducer,
    settingsImport: settingsImportReducer,
  };

  const rootReducer = combineReducers(reducers);

  return createStore(rootReducer, composeEnhancers());
}

function composeEnhancers(): StoreEnhancer {
  const actionCreators = {
    ...accountActions,
    ...connectionActions,
    ...downloadUpdateActions,
    ...settingsActions,
    ...supportActions,
    ...versionActions,
    ...userInterfaceActions,
  };

  if (window.env.development) {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const devtoolsCompose = (window as any).__REDUX_DEVTOOLS_EXTENSION_COMPOSE__?.({
      actionCreators,
    });
    return devtoolsCompose ? devtoolsCompose() : compose();
  }

  return compose();
}

// This hook adds type to state to make use simpler. It also prevents the state from update if the
// WillExit context value is true.
export function useSelector<R>(fn: (state: IReduxState) => R): R {
  const value = useReduxSelector(fn);
  const valueBeforeExit = useRef(value);
  const willExit = useWillExit();

  if (!willExit) {
    // eslint-disable-next-line react-compiler/react-compiler
    valueBeforeExit.current = value;
  }

  // eslint-disable-next-line react-compiler/react-compiler
  return valueBeforeExit.current;
}
