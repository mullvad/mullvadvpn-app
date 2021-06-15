import * as React from 'react';
import { Route, RouteComponentProps, Switch, withRouter } from 'react-router';
import Launch from './components/Launch';
import KeyboardNavigation from './components/KeyboardNavigation';
import MainView from './components/MainView';
import Focus, { IFocusHandle } from './components/Focus';
import LinuxSplitTunnelingSettings from './components/LinuxSplitTunnelingSettings';
import TransitionContainer, { TransitionView } from './components/TransitionContainer';
import AccountPage from './containers/AccountPage';
import AdvancedSettingsPage from './containers/AdvancedSettingsPage';
import LoginPage from './containers/LoginPage';
import PlatformWindowContainer from './containers/PlatformWindowContainer';
import PreferencesPage from './containers/PreferencesPage';
import SelectLanguagePage from './containers/SelectLanguagePage';
import SelectLocationPage from './containers/SelectLocationPage';
import SettingsPage from './containers/SettingsPage';
import SupportPage from './containers/SupportPage';
import WireguardKeysPage from './containers/WireguardKeysPage';
import History from './lib/history';
import { getTransitionProps } from './transitions';
import {
  SetupFinished,
  TimeAdded,
  VoucherInput,
  VoucherVerificationSuccess,
} from './components/ExpiredAccountAddTime';

interface IAppRoutesState {
  previousLocation?: RouteComponentProps['location'];
  currentLocation: RouteComponentProps['location'];
}

class AppRoutes extends React.Component<RouteComponentProps, IAppRoutesState> {
  private unobserveHistory?: () => void;

  private focusRef = React.createRef<IFocusHandle>();

  constructor(props: RouteComponentProps) {
    super(props);

    this.state = {
      currentLocation: props.location,
    };
  }

  public componentDidMount() {
    // React throttles updates, so it's impossible to capture the intermediate navigation without
    // listening to the history directly.
    this.unobserveHistory = (this.props.history as History).listen(
      (location, _action, affectedEntries) => {
        this.setState({
          previousLocation: affectedEntries[0],
          currentLocation: location,
        });
      },
    );
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

    return (
      <PlatformWindowContainer>
        <KeyboardNavigation>
          <Focus ref={this.focusRef}>
            <TransitionContainer onTransitionEnd={this.onNavigation} {...transitionProps}>
              <TransitionView viewId={location.key || ''}>
                <Switch key={location.key} location={location}>
                  <Route exact={true} path="/" component={Launch} />
                  <Route exact={true} path="/login" component={LoginPage} />
                  <Route exact={true} path="/main" component={MainView} />
                  <Route exact={true} path="/main/voucher/redeem" component={VoucherInput} />
                  <Route
                    exact={true}
                    path="/main/voucher/success"
                    component={VoucherVerificationSuccess}
                  />
                  <Route exact={true} path="/main/time-added" component={TimeAdded} />
                  <Route exact={true} path="/main/setup-finished" component={SetupFinished} />
                  <Route exact={true} path="/settings" component={SettingsPage} />
                  <Route exact={true} path="/settings/language" component={SelectLanguagePage} />
                  <Route exact={true} path="/settings/account" component={AccountPage} />
                  <Route exact={true} path="/settings/preferences" component={PreferencesPage} />
                  <Route exact={true} path="/settings/advanced" component={AdvancedSettingsPage} />
                  <Route
                    exact={true}
                    path="/settings/advanced/wireguard-keys"
                    component={WireguardKeysPage}
                  />
                  <Route
                    exact={true}
                    path="/settings/advanced/linux-split-tunneling"
                    component={LinuxSplitTunnelingSettings}
                  />
                  <Route exact={true} path="/settings/support" component={SupportPage} />
                  <Route exact={true} path="/select-location" component={SelectLocationPage} />
                </Switch>
              </TransitionView>
            </TransitionContainer>
          </Focus>
        </KeyboardNavigation>
      </PlatformWindowContainer>
    );
  }

  private onNavigation = () => {
    this.focusRef.current?.resetFocus();
  };
}

const AppRoutesWithRouter = withRouter(AppRoutes);

export default AppRoutesWithRouter;
