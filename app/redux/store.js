// @flow

import { createStore, applyMiddleware, combineReducers, compose } from 'redux';
import { routerMiddleware, connectRouter, push, replace } from 'connected-react-router';
import thunk from 'redux-thunk';

import account from './account/reducers';
import accountActions from './account/actions';
import connection from './connection/reducers';
import connectionActions from './connection/actions';
import settings from './settings/reducers';
import settingsActions from './settings/actions';
import support from './support/reducers';
import supportActions from './support/actions';
import daemon from './daemon/reducers';
import daemonActions from './daemon/actions';

import type { Store, StoreEnhancer } from 'redux';
import type { History } from 'history';
import type { AccountReduxState } from './account/reducers';
import type { ConnectionReduxState } from './connection/reducers';
import type { SettingsReduxState } from './settings/reducers';
import type { SupportReduxState } from './support/reducers';
import type { DaemonReduxState } from './daemon/reducers';

import type { AccountAction } from './account/actions';
import type { ConnectionAction } from './connection/actions';
import type { SettingsAction } from './settings/actions';
import type { SupportAction } from './support/actions';
import type { DaemonAction } from './daemon/actions';

export type ReduxState = {
  account: AccountReduxState,
  connection: ConnectionReduxState,
  settings: SettingsReduxState,
  support: SupportReduxState,
  daemon: DaemonReduxState,
};

export type ReduxAction =
  | AccountAction
  | ConnectionAction
  | SettingsAction
  | SupportAction
  | DaemonAction;
export type ReduxStore = Store<ReduxState, ReduxAction, ReduxDispatch>;
export type ReduxGetState = () => ReduxState;
export type ReduxDispatch = (action: ReduxAction | ReduxThunk) => any;
export type ReduxThunk = (dispatch: ReduxDispatch, getState: ReduxGetState) => any;

export default function configureStore(
  initialState: ?ReduxState,
  routerHistory: History,
): ReduxStore {
  const router = routerMiddleware(routerHistory);

  const actionCreators: { [string]: Function } = {
    ...accountActions,
    ...connectionActions,
    ...settingsActions,
    ...supportActions,
    ...daemonActions,
    pushRoute: (route) => push(route),
    replaceRoute: (route) => replace(route),
  };

  const reducers = {
    account,
    connection,
    settings,
    support,
    daemon,
  };

  const middlewares = [thunk, router];

  const composeEnhancers = (() => {
    const reduxCompose = window && window.__REDUX_DEVTOOLS_EXTENSION_COMPOSE__;
    if (process.env.NODE_ENV === 'development' && reduxCompose) {
      return reduxCompose({ actionCreators });
    }
    return compose;
  })();

  const enhancer: StoreEnhancer<ReduxState, ReduxAction, ReduxDispatch> = composeEnhancers(
    applyMiddleware(...middlewares),
  );
  const rootReducer = combineReducers(reducers);
  const rootReducerWithRouter = connectRouter(routerHistory)(rootReducer);
  if (initialState) {
    return createStore(rootReducerWithRouter, initialState, enhancer);
  } else {
    return createStore(rootReducerWithRouter, enhancer);
  }
}
