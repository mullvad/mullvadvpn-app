import { createStore, applyMiddleware, combineReducers, compose } from 'redux';
import { routerMiddleware, routerReducer as routing, push, replace } from 'react-router-redux';
import persistState from 'redux-localstorage';
import thunk from 'redux-thunk';

import user from './reducers/user';
import connect from './reducers/connect';
import settings from './reducers/settings';
import userActions from './actions/user';
import connectActions from './actions/connect';
import settingsActions from './actions/settings';

/**
 * Configure redux store
 * 
 * @export
 * @param {Object} initialState 
 * @param {History} routerHistory 
 * @returns {Redux.Store}
 */
export default function configureStore(initialState, routerHistory) {
  const router = routerMiddleware(routerHistory);
  
  const actionCreators = {
    ...userActions,
    ...connectActions,
    ...settingsActions,
    pushRoute: (route) => push(route),
    replaceRoute: (route) => replace(route),
  };

  const reducers = {
    user, connect, settings, routing
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
  
  return createStore(rootReducer, initialState, enhancer);
}
