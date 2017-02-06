import { createStore, applyMiddleware, combineReducers, compose } from 'redux';
import { hashHistory } from 'react-router';
import { routerMiddleware, routerReducer as routing, push } from 'react-router-redux';
import persistState from 'redux-localstorage';
import thunk from 'redux-thunk';

import user from './reducers/user';
import userActions from './actions/user';

const router = routerMiddleware(hashHistory);

const actionCreators = {
  ...userActions,
  push
};

const reducers = {
  user,
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
