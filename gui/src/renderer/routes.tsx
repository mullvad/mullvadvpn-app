import * as React from 'react';
import { Route, RouteComponentProps, Switch } from 'react-router';
import App from './app';
import TransitionContainer from './components/TransitionContainer';
import AccountPage from './containers/AccountPage';
import AdvancedSettingsPage from './containers/AdvancedSettingsPage';
import ConnectPage from './containers/ConnectPage';
import LaunchPage from './containers/LaunchPage';
import LoginPage from './containers/LoginPage';
import PlatformWindowContainer from './containers/PlatformWindowContainer';
import PreferencesPage from './containers/PreferencesPage';
import SelectLocationPage from './containers/SelectLocationPage';
import SettingsPage from './containers/SettingsPage';
import SupportPage from './containers/SupportPage';
import { getTransitionProps } from './transitions';

export interface ISharedRouteProps {
  app: App;
}

type CustomRouteProps = {
  component: React.ComponentClass<ISharedRouteProps>;
} & Route['props'];

export default function makeRoutes(componentProps: ISharedRouteProps) {
  // Renders a route extended with shared props
  function CustomRoute({ component: ComponentClass, ...routeProps }: CustomRouteProps) {
    const renderOverride = () => <ComponentClass {...componentProps} />;

    return <Route {...routeProps} render={renderOverride} />;
  }

  // store previous route
  let sourceRoute: string | null = null;

  function renderRoute({ location }: RouteComponentProps) {
    const destinationRoute = location.pathname;
    const transitionProps = getTransitionProps(sourceRoute, destinationRoute);
    sourceRoute = destinationRoute;

    return (
      <PlatformWindowContainer>
        <TransitionContainer {...transitionProps}>
          <Switch key={location.key} location={location}>
            <CustomRoute exact={true} path="/" component={LaunchPage} />
            <CustomRoute exact={true} path="/login" component={LoginPage} />
            <CustomRoute exact={true} path="/connect" component={ConnectPage} />
            <CustomRoute exact={true} path="/settings" component={SettingsPage} />
            <CustomRoute exact={true} path="/settings/account" component={AccountPage} />
            <CustomRoute exact={true} path="/settings/preferences" component={PreferencesPage} />
            <CustomRoute exact={true} path="/settings/advanced" component={AdvancedSettingsPage} />
            <CustomRoute exact={true} path="/settings/support" component={SupportPage} />
            <CustomRoute exact={true} path="/select-location" component={SelectLocationPage} />
          </Switch>
        </TransitionContainer>
      </PlatformWindowContainer>
    );
  }

  return <Route render={renderRoute} />;
}
