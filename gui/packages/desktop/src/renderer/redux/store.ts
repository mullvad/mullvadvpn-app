import { createStore, applyMiddleware, combineReducers, compose, Dispatch, Store } from 'redux';
import { routerMiddleware, connectRouter, push, replace } from 'connected-react-router';

import accountReducer from './account/reducers';
import accountActions from './account/actions';
import connectionReducer from './connection/reducers';
import connectionActions from './connection/actions';
import settingsReducer from './settings/reducers';
import settingsActions from './settings/actions';
import supportReducer from './support/reducers';
import supportActions from './support/actions';
import versionReducer from './version/reducers';
import versionActions from './version/actions';
import userInterfaceReducer from './userinterface/reducers';
import userInterfaceActions from './userinterface/actions';

import { History } from 'history';
import { AccountReduxState } from './account/reducers';
import { ConnectionReduxState } from './connection/reducers';
import { SettingsReduxState } from './settings/reducers';
import { SupportReduxState } from './support/reducers';
import { VersionReduxState } from './version/reducers';
import { UserInterfaceReduxState } from './userinterface/reducers';

import { AccountAction } from './account/actions';
import { ConnectionAction } from './connection/actions';
import { SettingsAction } from './settings/actions';
import { SupportAction } from './support/actions';
import { VersionAction } from './version/actions';
import { UserInterfaceAction } from './userinterface/actions';

export type ReduxState = {
  account: AccountReduxState;
  connection: ConnectionReduxState;
  settings: SettingsReduxState;
  support: SupportReduxState;
  version: VersionReduxState;
  userInterface: UserInterfaceReduxState;
};

export type ReduxAction =
  | AccountAction
  | ConnectionAction
  | SettingsAction
  | SupportAction
  | VersionAction
  | UserInterfaceAction;
export type ReduxStore = Store<ReduxState, ReduxAction>;
export type ReduxDispatch = Dispatch<ReduxAction>;

export default function configureStore(
  initialState: ReduxState | null,
  routerHistory: History,
): ReduxStore {
  const actionCreators: { [key: string]: Function } = {
    ...accountActions,
    ...connectionActions,
    ...settingsActions,
    ...supportActions,
    ...versionActions,
    ...userInterfaceActions,
    pushRoute: (route: string) => push(route),
    replaceRoute: (route: string) => replace(route),
  };

  const reducers = {
    account: accountReducer,
    connection: connectionReducer,
    settings: settingsReducer,
    support: supportReducer,
    version: versionReducer,
    userInterface: userInterfaceReducer,
    router: connectRouter(routerHistory),
  };

  const composeEnhancers: typeof compose = (() => {
    const reduxCompose = window && (window as any).__REDUX_DEVTOOLS_EXTENSION_COMPOSE__;
    if (process.env.NODE_ENV === 'development' && reduxCompose) {
      return reduxCompose({ actionCreators });
    }
    return compose;
  })();

  const enhancer = composeEnhancers(applyMiddleware(routerMiddleware(routerHistory)));

  const rootReducer = combineReducers(reducers);

  if (initialState) {
    return createStore(rootReducer, initialState, enhancer);
  } else {
    return createStore(rootReducer, enhancer);
  }
}
