import * as React from 'react';
import { Route, RouteComponentProps, RouteProps, Switch, withRouter } from 'react-router';
import App from './app';
import TransitionContainer, { TransitionView } from './components/TransitionContainer';
import AccountPage from './containers/AccountPage';
import AdvancedSettingsPage from './containers/AdvancedSettingsPage';
import ConnectPage from './containers/ConnectPage';
import LaunchPage from './containers/LaunchPage';
import LoginPage from './containers/LoginPage';
import PlatformWindowContainer from './containers/PlatformWindowContainer';
import PreferencesPage from './containers/PreferencesPage';
import SelectLanguagePage from './containers/SelectLanguagePage';
import SelectLocationPage from './containers/SelectLocationPage';
import SettingsPage from './containers/SettingsPage';
import SupportPage from './containers/SupportPage';
import WireguardKeysPage from './containers/WireguardKeysPage';
import { getTransitionProps } from './transitions';

export interface ISharedRouteProps {
  app: App;
}

type CustomRouteProps = {
  component: React.ComponentClass<ISharedRouteProps>;
} & RouteProps;

interface IAppRoutesProps extends RouteComponentProps {
  sharedProps: ISharedRouteProps;
}

interface IAppRoutesState {
  previousLocation?: IAppRoutesProps['location'];
  currentLocation: IAppRoutesProps['location'];
}

class AppRoutes extends React.Component<IAppRoutesProps, IAppRoutesState> {
  private unobserveHistory?: () => void;

  constructor(props: IAppRoutesProps) {
    super(props);

    this.state = {
      currentLocation: props.location,
    };
  }

  public componentDidMount() {
    // React throttles updates, so it's impossible to capture the intermediate navigation without
    // listening to the history directly.
    this.unobserveHistory = this.props.history.listen((location) => {
      this.setState((state) => ({
        previousLocation: state.currentLocation,
        currentLocation: location,
      }));
    });
  }

  public componentWillUnmount() {
    if (this.unobserveHistory) {
      this.unobserveHistory();
    }
  }

  public render() {
    const location = this.state.currentLocation;
    const transitionProps = getTransitionProps(
      this.state.previousLocation ? this.state.previousLocation.pathname : null,
      location.pathname,
    );

    // Renders a route extended with shared props
    const CustomRoute = ({ component: ComponentClass, ...routeProps }: CustomRouteProps) => {
      const renderOverride = () => <ComponentClass {...this.props.sharedProps} />;

      return <Route {...routeProps} render={renderOverride} />;
    };

    return (
      <PlatformWindowContainer>
        <TransitionContainer {...transitionProps}>
          <TransitionView viewId={location.key || ''}>
            <Switch key={location.key} location={location}>
              <CustomRoute exact={true} path="/" component={LaunchPage} />
              <CustomRoute exact={true} path="/login" component={LoginPage} />
              <CustomRoute exact={true} path="/connect" component={ConnectPage} />
              <CustomRoute exact={true} path="/settings" component={SettingsPage} />
              <CustomRoute exact={true} path="/settings/language" component={SelectLanguagePage} />
              <CustomRoute exact={true} path="/settings/account" component={AccountPage} />
              <CustomRoute exact={true} path="/settings/preferences" component={PreferencesPage} />
              <CustomRoute
                exact={true}
                path="/settings/advanced"
                component={AdvancedSettingsPage}
              />
              <CustomRoute
                exact={true}
                path="/settings/advanced/wireguard-keys"
                component={WireguardKeysPage}
              />
              <CustomRoute exact={true} path="/settings/support" component={SupportPage} />
              <CustomRoute exact={true} path="/select-location" component={SelectLocationPage} />
            </Switch>
          </TransitionView>
        </TransitionContainer>
      </PlatformWindowContainer>
    );
  }
}

const AppRoutesWithRouter = withRouter(AppRoutes);

export default AppRoutesWithRouter;
