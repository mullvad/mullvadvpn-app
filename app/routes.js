import React from 'react';
import { Route, IndexRoute } from 'react-router';

import App from './containers/App';
import LoginPage from './containers/LoginPage';
import LoggedInPage from './containers/LoggedInPage';

export default (
  <Route path="/" component={App}>
    <IndexRoute component={LoginPage} />
    <Route path="loggedin" component={LoggedInPage} />
  </Route>
);
