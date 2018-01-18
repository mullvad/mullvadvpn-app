// @flow

import React from 'react';
import { Switch, Route, Redirect } from 'react-router';
import { CSSTransitionGroup } from 'react-transition-group';
import PlatformWindow from './components/PlatformWindow';
import LoginPage from './containers/LoginPage';
import ConnectPage from './containers/ConnectPage';
import SettingsPage from './containers/SettingsPage';
import AdvancedSettingsPage from './containers/AdvancedSettingsPage';
import AccountPage from './containers/AccountPage';
import PreferencesPage from './containers/PreferencesPage';
import SupportPage from './containers/SupportPage';
import SelectLocationPage from './containers/SelectLocationPage';
import { getTransitionProps } from './transitions';

import type { ReduxGetState } from './redux/store';
import type { Backend } from './lib/backend';

export type SharedRouteProps = {
  backend: Backend
};

type CustomRouteProps = {
  component: ReactClass<*>
};

export default function makeRoutes(getState: ReduxGetState, componentProps: SharedRouteProps): React.Element<*> {

  // Merge props and render component
  const renderMergedProps = (ComponentClass: ReactClass<*>, ...rest: Array<Object>): React.Element<*> => {
    const finalProps = Object.assign({}, componentProps, ...rest);
    return (
      <ComponentClass { ...finalProps } />
    );
  };

  // Renders public route
  // example: <PublicRoute path="/" component={ MyComponent } />
  const PublicRoute = ({ component, ...otherProps }: CustomRouteProps) => {
    return (
      <Route { ...otherProps } render={ (routeProps) => {
        return renderMergedProps(component, routeProps, otherProps);
      }} />
    );
  };

  // Renders protected route that requires authentication, otherwise redirects to /
  // example: <PrivateRoute path="/protected" component={ MyComponent } />
  const PrivateRoute = ({ component, ...otherProps }: CustomRouteProps) => {
    return (
      <Route { ...otherProps } render={ (routeProps) => {
        const { account } = getState();
        const isLoggedIn = account.status === 'ok';

        if(isLoggedIn) {
          return renderMergedProps(component, routeProps, otherProps);
        } else {
          return (<Redirect to={ '/' } />);
        }
      }} />
    );
  };

  // Renders login route that is only available to non-authenticated
  // users. Otherwise this route redirects user to /connect.
  // example: <LoginRoute path="/login" component={ MyComponent } />
  const LoginRoute = ({ component, ...otherProps }: CustomRouteProps) => {
    return (
      <Route { ...otherProps } render={ (routeProps) => {
        const { account } = getState();
        const isLoggedIn = account.status === 'ok';

        if(isLoggedIn) {
          return (<Redirect to={ '/connect' } />);
        } else {
          return renderMergedProps(component, routeProps, otherProps);
        }
      }} />
    );
  };

  // store previous route
  let previousRoute: ?string;

  return (
    <Route render={({ location }) => {
      const toRoute = location.pathname;
      const fromRoute = previousRoute;
      const transitionProps = getTransitionProps(fromRoute, toRoute);
      previousRoute = toRoute;

      return (
        <PlatformWindow>
          <CSSTransitionGroup component="div" className="transition-container" { ...transitionProps }>
            <Switch key={ location.key } location={ location }>
              <LoginRoute exact path="/" component={ LoginPage } />
              <PrivateRoute exact path="/connect" component={ ConnectPage } />
              <PublicRoute exact path="/settings" component={ SettingsPage } />
              <PrivateRoute exact path="/settings/account" component={ AccountPage } />
              <PublicRoute exact path="/settings/preferences" component={ PreferencesPage } />
              <PublicRoute exact path="/settings/advanced" component={ AdvancedSettingsPage } />
              <PublicRoute exact path="/settings/support" component={ SupportPage } />
              <PrivateRoute exact path="/select-location" component={ SelectLocationPage } />
            </Switch>
          </CSSTransitionGroup>
        </PlatformWindow>
      );
    }} />
  );
}
