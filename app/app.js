import React from 'react';
import ReactDOM from 'react-dom';
import { Provider } from 'react-redux';
import { Router, hashHistory } from 'react-router';
import { syncHistoryWithStore } from 'react-router-redux';
import { remote } from 'electron';
import path from 'path';
import routes from './routes';
import configureStore from './store';
import Tray from './containers/Tray';

const iconPath = path.join(__dirname, 'assets/trayicon.png');
const tray = new remote.Tray(iconPath);

const initialState = {};
const store = configureStore(initialState);
const routerHistory = syncHistoryWithStore(hashHistory, store);
const rootElement = document.querySelector(document.currentScript.getAttribute('data-container'));

ReactDOM.render(
  <div>
    <Provider store={store}>
      <Router history={routerHistory} routes={routes} />
    </Provider>
    <Provider store={store}>
      <Tray handle={tray} history={routerHistory} />
    </Provider>
  </div>,
  rootElement
);
