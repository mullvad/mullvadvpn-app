// @flow

import * as React from 'react';
import { Switch, Route, Redirect } from 'react-router';
import TransitionContainer from './components/TransitionContainer';
import PlatformWindow from './components/PlatformWindow';
import LaunchPage from './containers/LaunchPage';
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
import type App from './app';

export type SharedRouteProps = {
  app: App,
};

export default function makeRoutes(
  getState: ReduxGetState,
  componentProps: SharedRouteProps,
): React.Element<*> {
  // Merge props and render component
  const renderMergedProps = (
    ComponentClass: React.ComponentType<*>,
    ...rest: Array<Object>
  ): React.Element<*> => {
    const finalProps = Object.assign({}, componentProps, ...rest);
    return <ComponentClass {...finalProps} />;
  };

  // Renders public route
  // example: <PublicRoute path="/" component={ MyComponent } />
  const PublicRoute = ({ component, ...otherProps }) => {
    return (
      <Route
        {...otherProps}
        render={(routeProps) => {
          return renderMergedProps(component, routeProps, otherProps);
        }}
      />
    );
  };

  // Renders protected route that requires authentication, otherwise redirects to /
  // example: <PrivateRoute path="/protected" component={ MyComponent } />
  const PrivateRoute = ({ component, ...otherProps }) => {
    return (
      <Route
        {...otherProps}
        render={(routeProps) => {
          const { account } = getState();
          const isLoggedIn = account.status === 'ok';

          if (isLoggedIn) {
            return renderMergedProps(component, routeProps, otherProps);
          } else {
            return <Redirect to={'/login'} />;
          }
        }}
      />
    );
  };

  // Renders login route that is only available to non-authenticated
  // users. Otherwise this route redirects user to /connect.
  // example: <LoginRoute path="/login" component={ MyComponent } />
  const LoginRoute = ({ component, ...otherProps }) => {
    return (
      <Route
        {...otherProps}
        render={(routeProps) => {
          const { account } = getState();
          const isLoggedIn = account.status === 'ok';

          if (isLoggedIn) {
            return <Redirect to={'/connect'} />;
          } else {
            return renderMergedProps(component, routeProps, otherProps);
          }
        }}
      />
    );
  };

  // Renders launch route that is only available when daemon is not connected.
  // Otherwise this route redirects user to /login.
  // example: <LaunchRoute path="/" component={ MyComponent } />
  const LaunchRoute = ({ component, ...otherProps }) => {
    return (
      <Route
        {...otherProps}
        render={(routeProps) => {
          const { daemon } = getState();
          if (daemon.isConnected) {
            return <Redirect to={'/login'} />;
          } else {
            return renderMergedProps(component, routeProps, otherProps);
          }
        }}
      />
    );
  };

  // store previous route
  let previousRoute: ?string;

  return (
    <Route
      render={({ location }) => {
        const toRoute = location.pathname;
        const fromRoute = previousRoute;
        const transitionProps = getTransitionProps(fromRoute, toRoute);
        previousRoute = toRoute;

        return (
          <PlatformWindow>
            <TransitionContainer {...transitionProps}>
              <Switch key={location.key} location={location}>
                <LaunchRoute exact path="/" component={LaunchPage} />
                <LoginRoute exact path="/login" component={LoginPage} />
                <PrivateRoute exact path="/connect" component={ConnectPage} />
                <PublicRoute exact path="/settings" component={SettingsPage} />
                <PrivateRoute exact path="/settings/account" component={AccountPage} />
                <PublicRoute exact path="/settings/preferences" component={PreferencesPage} />
                <PublicRoute exact path="/settings/advanced" component={AdvancedSettingsPage} />
                <PublicRoute exact path="/settings/support" component={SupportPage} />
                <PrivateRoute exact path="/select-location" component={SelectLocationPage} />
              </Switch>
            </TransitionContainer>
          </PlatformWindow>
        );
      }}
    />
  );
}
