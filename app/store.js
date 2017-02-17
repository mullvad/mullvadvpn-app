import { createStore, applyMiddleware, combineReducers, compose } from 'redux';
import { hashHistory } from 'react-router';
import { routerMiddleware, routerReducer as routing, push, replace } from 'react-router-redux';
import persistState from 'redux-localstorage';
import thunk from 'redux-thunk';

import user from './reducers/user';
import connect from './reducers/connect';
import settings from './reducers/settings';
import userActions from './actions/user';
import connectActions from './actions/connect';
import settingsActions from './actions/settings';

const router = routerMiddleware(hashHistory);

const actionCreators = {
  ...userActions,
  ...connectActions,
  ...settingsActions,
  pushRoute: (route) => push(route),
  replaceRoute: (route) => replace(route),
};

const reducers = {
  user,
  connect,
  settings,
  routing
};

const middlewares = [ thunk, router ];

const composeEnhancers = (() => {
  const compose_ = window && window.__REDUX_DEVTOOLS_EXTENSION_COMPOSE__;
  if(process.env.NODE_ENV === 'development' && compose_) {
    return compose_({ actionCreators });
  }
  return compose;
})();

export default function configureStore(initialState) {
  const enhancer = composeEnhancers(applyMiddleware(...middlewares), persistState());
  const rootReducer = combineReducers(reducers);
  
  return createStore(rootReducer, initialState, enhancer);
}
