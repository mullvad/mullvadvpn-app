import React from 'react';
import ReactDOM from 'react-dom';
import { Provider } from 'react-redux';
import { Router, hashHistory } from 'react-router';
import { syncHistoryWithStore } from 'react-router-redux';
import { remote, webFrame } from 'electron';
import path from 'path';
import routes from './routes';
import configureStore from './store';
import Tray from './containers/Tray';
import Backend from './lib/backend';

const backend = new Backend();

const iconPath = path.join(__dirname, 'assets/images/trayicon.png');
const tray = new remote.Tray(iconPath);

const initialState = {};
const store = configureStore(initialState);
const routerHistory = syncHistoryWithStore(hashHistory, store);
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
  <div>
    <Provider store={store}>
      <Router history={routerHistory} routes={routes} createElement={createElement} />
    </Provider>
    <Provider store={store}>
      <Tray handle={tray} history={routerHistory} />
    </Provider>
  </div>,
  rootElement
);
