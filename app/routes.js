import React from 'react';
import { Route, IndexRoute } from 'react-router';

import App from './containers/App';
import LoginPage from './containers/LoginPage';
import ConnectPage from './containers/ConnectPage';

export default (
  <Route path="/" component={ App }>
    <IndexRoute component={ LoginPage } />
    <Route path="connect" component={ ConnectPage } />
  </Route>
);
