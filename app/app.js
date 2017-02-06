import React from 'react';
import ReactDOM from 'react-dom';
import { Provider } from 'react-redux';
import { Router, hashHistory } from 'react-router';
import { syncHistoryWithStore } from 'react-router-redux';
import routes from './routes';
import configureStore from './store';

const initialState = {};
const store = configureStore(initialState);
const routerHistory = syncHistoryWithStore(hashHistory, store);

const rootElement = document.querySelector(document.currentScript.getAttribute('data-container'));

ReactDOM.render(
  <Provider store={store}>
    <Router history={routerHistory} routes={routes} />
  </Provider>,
  rootElement
);
