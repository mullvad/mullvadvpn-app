import React from 'react';
import { Switch, Route, Redirect } from 'react-router';
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
 * @param {function} getState       - function to get redux state
 * @param {object}   componentProps - extra props to propagate across components
 * @returns {React.element}
 */
export default function makeRoutes(getState, componentProps) {

  /**
   * Merge props and render component
   * @param {React.Component} component - component class
   * @param {...}             rest      - props
   * @private
   */
  const renderMergedProps = (component, ...rest) => {
    const finalProps = Object.assign({}, componentProps, ...rest);
    return (
      React.createElement(component, finalProps)
    );
  };

  /**
   * Renders public route
   * Example: <PublicRoute path="/" component={ MyComponent } />
   * @private
   */
  const PublicRoute = ({ component, ...rest }) => {
    return (
      <Route {...rest} render={ (routeProps) => {
        return renderMergedProps(component, routeProps, ...rest);
      }} />
    );
  };

  /**
   * Renders protected route that requires authentication, otherwise redirects to /
   * Example: <PrivateRoute path="/protected" component={ MyComponent } />
   * @private
   */
  const PrivateRoute = ({ component, ...rest }) => {
    return (
      <Route {...rest} render={ (routeProps) => {
        const { user } = getState();
        const isLoggedIn = user.status === LoginState.ok;

        if(isLoggedIn) {
          return renderMergedProps(component, routeProps, ...rest);
        } else {
          return (<Redirect to={ '/' } />);
        }
      }} />
    );
  };

  /**
   * Renders login route that is only available to non-authenticated
   * users. Otherwise this route redirects user to /connect.
   * Example: <LoginRoute path="/login" component={ MyComponent } />
   * @private
   */
  const LoginRoute = ({ component, ...rest }) => {
    return (
      <Route {...rest} render={ (routeProps) => {
        const { user } = getState();
        const isLoggedIn = user.status === LoginState.ok;

        if(isLoggedIn) {
          return (<Redirect to={ '/connect' } />);
        } else {
          return renderMergedProps(component, routeProps, ...rest);
        }
      }} />
    );
  };

  return (
    <Switch>
      <LoginRoute exact path="/" component={ LoginPage } />
      <PrivateRoute exact path="/connect" component={ ConnectPage } />
      <PublicRoute exact path="/settings" component={ SettingsPage } />
      <PrivateRoute path="/settings/account" component={ AccountPage } />
      <PrivateRoute path="/select-location" component={ SelectLocationPage } />
    </Switch>
  );
}
