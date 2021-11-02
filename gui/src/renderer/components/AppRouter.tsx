import { Action } from 'history';
import * as React from 'react';
import { Route, Switch } from 'react-router';
import Launch from './Launch';
import KeyboardNavigation from './KeyboardNavigation';
import MainView from './MainView';
import Focus, { IFocusHandle } from './Focus';
import SplitTunnelingSettings from './SplitTunnelingSettings';
import TransitionContainer, { TransitionView } from './TransitionContainer';
import AccountPage from '../containers/AccountPage';
import AdvancedSettingsPage from '../containers/AdvancedSettingsPage';
import LoginPage from '../containers/LoginPage';
import OpenVPNSettingsPage from '../containers/OpenVPNSettingsPage';
import PreferencesPage from '../containers/PreferencesPage';
import SelectLanguagePage from '../containers/SelectLanguagePage';
import SelectLocationPage from '../containers/SelectLocationPage';
import SettingsPage from '../containers/SettingsPage';
import SupportPage from '../containers/SupportPage';
import WireguardSettingsPage from '../containers/WireguardSettingsPage';
import { IHistoryProps, ITransitionSpecification, transitions, withHistory } from '../lib/history';
import {
  SetupFinished,
  TimeAdded,
  VoucherInput,
  VoucherVerificationSuccess,
} from './ExpiredAccountAddTime';
import { RoutePath } from '../lib/routes';
import FilterByProvider from './FilterByProvider';

interface IAppRoutesState {
  currentLocation: IHistoryProps['history']['location'];
  transition: ITransitionSpecification;
  action?: Action;
}

class AppRouter extends React.Component<IHistoryProps, IAppRoutesState> {
  private unobserveHistory?: () => void;

  private focusRef = React.createRef<IFocusHandle>();

  constructor(props: IHistoryProps) {
    super(props);

    this.state = {
      currentLocation: props.history.location,
      transition: transitions.none,
    };
  }

  public componentDidMount() {
    // React throttles updates, so it's impossible to capture the intermediate navigation without
    // listening to the history directly.
    this.unobserveHistory = this.props.history.listen((location, action, transition) => {
      this.setState({
        currentLocation: location,
        transition,
        action,
      });
    });
  }

  public componentWillUnmount() {
    if (this.unobserveHistory) {
      this.unobserveHistory();
    }
  }

  public render() {
    const location = this.state.currentLocation;

    return (
      <KeyboardNavigation>
        <Focus ref={this.focusRef}>
          <TransitionContainer onTransitionEnd={this.onNavigation} {...this.state.transition}>
            <TransitionView viewId={location.key || ''}>
              <Switch key={location.key} location={location}>
                <Route exact path={RoutePath.launch} component={Launch} />
                <Route exact path={RoutePath.login} component={LoginPage} />
                <Route exact path={RoutePath.main} component={MainView} />
                <Route exact path={RoutePath.redeemVoucher} component={VoucherInput} />
                <Route
                  exact
                  path={RoutePath.voucherSuccess}
                  component={VoucherVerificationSuccess}
                />
                <Route exact path={RoutePath.timeAdded} component={TimeAdded} />
                <Route exact path={RoutePath.setupFinished} component={SetupFinished} />
                <Route exact path={RoutePath.settings} component={SettingsPage} />
                <Route exact path={RoutePath.selectLanguage} component={SelectLanguagePage} />
                <Route exact path={RoutePath.accountSettings} component={AccountPage} />
                <Route exact path={RoutePath.preferences} component={PreferencesPage} />
                <Route exact path={RoutePath.advancedSettings} component={AdvancedSettingsPage} />
                <Route exact path={RoutePath.wireguardSettings} component={WireguardSettingsPage} />
                <Route exact path={RoutePath.openVpnSettings} component={OpenVPNSettingsPage} />
                <Route exact path={RoutePath.splitTunneling} component={SplitTunnelingSettings} />
                <Route exact path={RoutePath.support} component={SupportPage} />
                <Route exact path={RoutePath.selectLocation} component={SelectLocationPage} />
                <Route exact path={RoutePath.filterByProvider} component={FilterByProvider} />
              </Switch>
            </TransitionView>
          </TransitionContainer>
        </Focus>
      </KeyboardNavigation>
    );
  }

  private onNavigation = () => {
    this.focusRef.current?.resetFocus();
  };
}

const AppRoutesWithRouter = withHistory(AppRouter);

export default AppRoutesWithRouter;
