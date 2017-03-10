import React from 'react';
import { Route, IndexRoute } from 'react-router';

import App from './containers/App';
import LoginPage from './containers/LoginPage';
import ConnectPage from './containers/ConnectPage';
import SettingsPage from './containers/SettingsPage';
import AccountPage from './containers/AccountPage';
import SelectLocationPage from './containers/SelectLocationPage';

import { LoginState } from './enums';

/**
 * Create routes
 * 
 * @export
 * @param {Redux.Store} store 
 * @returns {React.element}
 */
export default function makeRoutes(store) {

  /**
   * Ensures that user is redirected to /connect if logged in
   */
  const ensureConnect = (nextState, replace) => {
    let { user } = store.getState();
    if(user.status === LoginState.ok) {
      replace('/connect');
    }
  };

  /**
   * Ensures that user is redirected to / login if not logged in
   */
  const ensureLoggedIn = (nextState, replace) => {
    let { user } = store.getState();
    if(user.status !== LoginState.ok) {
      replace('/');
    }
  };

  return (
    <Route path="/" component={ App }>
      <IndexRoute component={ LoginPage } onEnter={ ensureConnect } />
      <Route path="connect" component={ ConnectPage } onEnter={ ensureLoggedIn } />
      <Route path="settings">
        <IndexRoute component={ SettingsPage } />
        <Route path="account" component={ AccountPage } onEnter={ ensureLoggedIn } />
      </Route>
      <Route path="select-location" component={ SelectLocationPage } onEnter={ ensureLoggedIn } />
    </Route>
  );
}
