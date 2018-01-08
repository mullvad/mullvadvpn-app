// @flow

import React from 'react';
import RX, { Component, Text } from 'reactxp';
import { Provider } from 'react-redux';
import { Router } from 'react-router-native';
import { createMemoryHistory } from 'history';
import makeRoutes from './js/routes';
import configureStore from './js/redux/store';
import { Backend } from './js/lib/backend';
import { DeviceEventEmitter } from 'react-native';
import { MobileAppBridge } from 'NativeModules';
import { Dimensions } from 'react-native';

import type { ConnectionState } from './js/redux/connection/reducers';
import type { TrayIconType } from './js/lib/tray-icon-manager';

const initialState = null;
const memoryHistory = createMemoryHistory();
const store = configureStore(initialState, memoryHistory);

//////////////////////////////////////////////////////////////////////////
// Backend
//////////////////////////////////////////////////////////////////////////
const backend = new Backend(store);

DeviceEventEmitter.addListener('com.mullvad.backend-info', function(e: Event) {
  console.warn(e);
  backend.init();
  backend.sync();
  backend.autologin();
});

MobileAppBridge.startBackend().then(response => {})
  .catch(e => {
    console.warn('Failed starting backend:', e);
  });

const _isPortrait = () => {
  const dim = RX.UserInterface.measureWindow();
  return dim.height >= dim.width;
};

export default class App extends Component{

  constructor() {
      super();

      this.state = {
          orientation: _isPortrait() ? 'portrait' : 'landscape',
      };

      Dimensions.addEventListener('change', () => {
          this.setState({
              orientation: _isPortrait() ? 'portrait' : 'landscape'
          });
      });
  }

  componentWillMount() {

  }

  render() {
    return (
      <Provider store={ store }>
      <Router history={ memoryHistory }>
      { makeRoutes(store.getState, { backend }) }
      </Router>
      </Provider>
      );
  }
}
