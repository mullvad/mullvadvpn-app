import * as React from 'react';
import { Switch, Route } from 'react-router';
import TransitionContainer from './components/TransitionContainer';
import PlatformWindowContainer from './containers/PlatformWindowContainer';
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

import App from './app';
export type SharedRouteProps = {
  app: App;
};

type CustomRouteProps = {
  component: React.ComponentClass<SharedRouteProps>;
} & Route['props'];

export default function makeRoutes(componentProps: SharedRouteProps) {
  // Renders a route extended with shared props
  const CustomRoute = ({ component: ComponentClass, ...routeProps }: CustomRouteProps) => (
    <Route {...routeProps} render={() => <ComponentClass {...componentProps} />} />
  );

  // store previous route
  let sourceRoute: string | null = null;

  return (
    <Route
      render={({ location }) => {
        const destinationRoute = location.pathname;
        const transitionProps = getTransitionProps(sourceRoute, destinationRoute);
        sourceRoute = destinationRoute;

        return (
          <PlatformWindowContainer>
            <TransitionContainer {...transitionProps}>
              <Switch key={location.key} location={location}>
                <CustomRoute exact path="/" component={LaunchPage} />
                <CustomRoute exact path="/login" component={LoginPage} />
                <CustomRoute exact path="/connect" component={ConnectPage} />
                <CustomRoute exact path="/settings" component={SettingsPage} />
                <CustomRoute exact path="/settings/account" component={AccountPage} />
                <CustomRoute exact path="/settings/preferences" component={PreferencesPage} />
                <CustomRoute exact path="/settings/advanced" component={AdvancedSettingsPage} />
                <CustomRoute exact path="/settings/support" component={SupportPage} />
                <CustomRoute exact path="/select-location" component={SelectLocationPage} />
              </Switch>
            </TransitionContainer>
          </PlatformWindowContainer>
        );
      }}
    />
  );
}
