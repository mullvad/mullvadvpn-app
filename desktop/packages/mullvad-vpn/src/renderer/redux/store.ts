import { useSelector as useReduxSelector } from 'react-redux';
import { combineReducers, compose, createStore, Dispatch, StoreEnhancer } from 'redux';

import { AppUpgradeAction, appUpgradeActions } from './app-upgrade/actions';
import { appUpgradeReducer, AppUpgradeReduxState } from './app-upgrade/reducers';
import connectionActions, { ConnectionAction } from './connection/actions';
import connectionReducer, { IConnectionReduxState } from './connection/reducers';
import identityActions, { IdentityAction } from './identity/actions';
import identityReducer, { IIdentityReduxState } from './identity/reducers';
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
  appUpgrade: AppUpgradeReduxState;
  connection: IConnectionReduxState;
  identity: IIdentityReduxState;
  settings: ISettingsReduxState;
  support: ISupportReduxState;
  version: IVersionReduxState;
  userInterface: IUserInterfaceReduxState;
  settingsImport: SettingsImportReduxState;
}

export type ReduxAction =
  | AppUpgradeAction
  | ConnectionAction
  | IdentityAction
  | SettingsAction
  | SupportAction
  | VersionAction
  | UserInterfaceAction
  | SettingsImportAction;
export type ReduxStore = ReturnType<typeof configureStore>;
export type ReduxDispatch = Dispatch<ReduxAction>;

export default function configureStore() {
  const reducers = {
    appUpgrade: appUpgradeReducer,
    connection: connectionReducer,
    identity: identityReducer,
    settings: settingsReducer,
    support: supportReducer,
    version: versionReducer,
    userInterface: userInterfaceReducer,
    settingsImport: settingsImportReducer,
  };

  const rootReducer = combineReducers(reducers);

  return createStore(rootReducer, {}, composeEnhancers());
}

function composeEnhancers(): StoreEnhancer {
  const actionCreators = {
    ...appUpgradeActions,
    ...connectionActions,
    ...identityActions,
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

// This hook adds type to state to make use simpler.
export function useSelector<R>(
  fn: (state: IReduxState) => R,
  equalityFn?: Parameters<typeof useReduxSelector>[1],
): R {
  return useReduxSelector(fn, equalityFn);
}
