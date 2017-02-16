import React from 'react';
import ReactDOM from 'react-dom';
import { Provider } from 'react-redux';
import { Router, hashHistory } from 'react-router';
import { syncHistoryWithStore } from 'react-router-redux';
import { webFrame } from 'electron';
import routes from './routes';
import configureStore from './store';
import Backend from './lib/backend';

const backend = new Backend();

const initialState = {};
const store = configureStore(initialState);

// desperately trying to fix https://github.com/reactjs/react-router-redux/issues/534
hashHistory.replace('/');

// see https://github.com/reactjs/react-router-redux/issues/534
const recentLocation = (store.getState().routing || {}).locationBeforeTransitions;
const routerHistory = syncHistoryWithStore(hashHistory, store, { adjustUrlOnReplay: true });

if(recentLocation && recentLocation.pathname) {
  routerHistory.replace(recentLocation.pathname);
}

const rootElement = document.querySelector(document.currentScript.getAttribute('data-container'));

// disable smart pinch.
webFrame.setVisualZoomLevelLimits(1, 1);

// helper method for router to pass backend down the component tree
const createElement = (Component, props) => {
  const newProps = { ...props, backend };
  return (
    <Component {...newProps} />
  );
};

ReactDOM.render(
  <Provider store={ store }>
    <Router history={ routerHistory } routes={ routes } createElement={ createElement } />
  </Provider>,
  rootElement
);
