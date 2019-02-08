import { connectRouter, push, replace, routerMiddleware } from 'connected-react-router';
import { applyMiddleware, combineReducers, compose, createStore, Dispatch, Store } from 'redux';

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

import { History } from 'history';

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
export type ReduxStore = Store<IReduxState, ReduxAction>;
export type ReduxDispatch = Dispatch<ReduxAction>;

export default function configureStore(
  routerHistory: History,
  initialState?: IReduxState,
): ReduxStore {
  const actionCreators = {
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
