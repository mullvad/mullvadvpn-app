// @flow
import { createStore, applyMiddleware, combineReducers, compose } from 'redux';
import { routerMiddleware, routerReducer, push, replace } from 'react-router-redux';
import persistState from 'redux-localstorage';
import thunk from 'redux-thunk';
import user from './reducers/user';
import connect from './reducers/connect';
import settings from './reducers/settings';
import userActions from './actions/user';
import connectActions from './actions/connect';
import settingsActions from './actions/settings';

import type { Store, Dispatch } from 'redux';
import type { History } from 'history';
import type { UserReduxState } from './reducers/user';
import type { ConnectReduxState } from './reducers/connect';
import type { SettingsReduxState } from './reducers/settings';

export type ReduxState = {
  user: UserReduxState,
  connect: ConnectReduxState,
  settings: SettingsReduxState
};
export type ReduxAction<T> = { type: string, payload: T };
export type ReduxStore = Store<ReduxState, ReduxAction<*>>;
export type ReduxGetStateFn = () => ReduxState;
export type ReduxDispatchFn<T: *> = Dispatch<ReduxAction<T>>;

export default function configureStore(initialState: ?ReduxState, routerHistory: History): ReduxStore {
  const router = routerMiddleware(routerHistory);

  const actionCreators: { string: Function } = {
    ...userActions,
    ...connectActions,
    ...settingsActions,
    pushRoute: (route) => push(route),
    replaceRoute: (route) => replace(route),
  };

  const reducers = {
    user, connect, settings, router: routerReducer
  };

  const middlewares = [ thunk, router ];

  const composeEnhancers = (() => {
    const reduxCompose = window && window.__REDUX_DEVTOOLS_EXTENSION_COMPOSE__;
    if(process.env.NODE_ENV === 'development' && reduxCompose) {
      return reduxCompose({ actionCreators });
    }
    return compose;
  })();

  const enhancer = composeEnhancers(applyMiddleware(...middlewares), persistState());
  const rootReducer = combineReducers(reducers);
  if(initialState) {
    return createStore(rootReducer, initialState, enhancer);
  }
  return createStore(rootReducer, enhancer);
}
