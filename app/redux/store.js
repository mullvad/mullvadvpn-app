// @flow
import { createStore, applyMiddleware, combineReducers, compose } from 'redux';
import { routerMiddleware, routerReducer, push, replace } from 'react-router-redux';
import thunk from 'redux-thunk';

import account from './account/reducers.js';
import accountActions from './account/actions.js';
import connection from './connection/reducers.js';
import connectionActions from './connection/actions.js';
import settings from './settings/reducers.js';
import settingsActions from './settings/actions.js';

import type { Store } from 'redux';
import type { History } from 'history';
import type { AccountReduxState } from './account/reducers.js';
import type { ConnectionReduxState } from './connection/reducers.js';
import type { SettingsReduxState } from './settings/reducers.js';

import type { ConnectionAction } from './connection/actions.js';
import type { AccountAction } from './account/actions.js';
import type { UpdateSettingsAction } from './settings/actions.js';

export type ReduxState = {
  account: AccountReduxState,
  connection: ConnectionReduxState,
  settings: SettingsReduxState
};

export type ReduxAction = AccountAction
                          | UpdateSettingsAction
                          | ConnectionAction;

export type ReduxStore = Store<ReduxState, ReduxAction, ReduxDispatch>;
export type ReduxGetState = () => ReduxState;
export type ReduxDispatch = (action: ReduxAction | ReduxThunk) => any;
export type ReduxThunk = (dispatch: ReduxDispatch, getState: ReduxGetState) => any;

export default function configureStore(initialState: ?ReduxState, routerHistory: History): ReduxStore {
  const router = routerMiddleware(routerHistory);

  const actionCreators: { [string]: Function } = {
    ...accountActions,
    ...connectionActions,
    ...settingsActions,
    pushRoute: (route) => push(route),
    replaceRoute: (route) => replace(route),
  };

  const reducers = {
    account, connection, settings, router: routerReducer
  };

  const middlewares = [ thunk, router ];

  const composeEnhancers = (() => {
    const reduxCompose = window && window.__REDUX_DEVTOOLS_EXTENSION_COMPOSE__;
    if(process.env.NODE_ENV === 'development' && reduxCompose) {
      return reduxCompose({ actionCreators });
    }
    return compose;
  })();

  const enhancer = composeEnhancers(applyMiddleware(...middlewares));
  const rootReducer = combineReducers(reducers);
  if(initialState) {
    return createStore(rootReducer, initialState, enhancer);
  }
  return createStore(rootReducer, enhancer);
}
