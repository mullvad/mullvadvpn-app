import { useRef } from 'react';
import { useSelector as useReduxSelector } from 'react-redux';
import { combineReducers, compose, createStore, Dispatch } from 'redux';

import { usePause } from '../lib/pause-rendering';
import accountActions, { AccountAction } from './account/actions';
import accountReducer, { IAccountReduxState } from './account/reducers';
import connectionActions, { ConnectionAction } from './connection/actions';
import connectionReducer, { IConnectionReduxState } from './connection/reducers';
import settingsActions, { SettingsAction } from './settings/actions';
import settingsReducer, { ISettingsReduxState } from './settings/reducers';
import supportActions, { SupportAction } from './support/actions';
import supportReducer, { ISupportReduxState } from './support/reducers';
import userInterfaceActions, { UserInterfaceAction } from './userinterface/actions';
import userInterfaceReducer, { IUserInterfaceReduxState } from './userinterface/reducers';
import versionActions, { VersionAction } from './version/actions';
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

export default function configureStore() {
  const reducers = {
    account: accountReducer,
    connection: connectionReducer,
    settings: settingsReducer,
    support: supportReducer,
    version: versionReducer,
    userInterface: userInterfaceReducer,
  };

  const rootReducer = combineReducers(reducers);

  return createStore(rootReducer, composeEnhancers());
}

function composeEnhancers(): typeof compose {
  const actionCreators = {
    ...accountActions,
    ...connectionActions,
    ...settingsActions,
    ...supportActions,
    ...versionActions,
    ...userInterfaceActions,
  };

  return window.env.development
    ? // eslint-disable-next-line @typescript-eslint/no-explicit-any
      (window as any).__REDUX_DEVTOOLS_EXTENSION_COMPOSE__({ actionCreators })()
    : compose();
}

// This hook adds typing to state to make use simpler. It also prevents the state from update if the
// ReduxPause context has been told to pause updates caused by new values in the redux state.
export function useSelector<R>(fn: (state: IReduxState) => R): R {
  const [paused] = usePause();
  const value = useReduxSelector(fn);
  const valueBeforePause = useRef(value);

  if (!paused) {
    valueBeforePause.current = value;
  }

  return paused ? valueBeforePause.current : value;
}
