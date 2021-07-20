import { Action } from 'history';
import * as React from 'react';
import { Route as ReactRouterRoute, Switch } from 'react-router';
import Launch from './Launch';
import KeyboardNavigation from './KeyboardNavigation';
import MainView from './MainView';
import Focus, { IFocusHandle } from './Focus';
import SplitTunnelingSettings from './SplitTunnelingSettings';
import TransitionContainer, { TransitionView } from './TransitionContainer';
import AccountPage from '../containers/AccountPage';
import AdvancedSettingsPage from '../containers/AdvancedSettingsPage';
import LoginPage from '../containers/LoginPage';
import PlatformWindowContainer from '../containers/PlatformWindowContainer';
import PreferencesPage from '../containers/PreferencesPage';
import SelectLanguagePage from '../containers/SelectLanguagePage';
import SelectLocationPage from '../containers/SelectLocationPage';
import SettingsPage from '../containers/SettingsPage';
import SupportPage from '../containers/SupportPage';
import WireguardKeysPage from '../containers/WireguardKeysPage';
import { IHistoryProps, ITransitionSpecification, transitions, withHistory } from '../lib/history';
import {
  SetupFinished,
  TimeAdded,
  VoucherInput,
  VoucherVerificationSuccess,
} from './ExpiredAccountAddTime';
import { RoutePath } from '../lib/routes';

interface IAppRoutesState {
  currentLocation: IHistoryProps['history']['location'];
  transition: ITransitionSpecification;
  action?: Action;
}

interface IRouteProps<T> {
  component: React.ComponentType<T>;
  path: RoutePath | RoutePath[];
  exact?: boolean;
}

function Route<T>(props: IRouteProps<T>) {
  return (
    <ReactRouterRoute path={props.path} exact={props.exact ?? true} component={props.component} />
  );
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
      <PlatformWindowContainer>
        <KeyboardNavigation>
          <Focus ref={this.focusRef}>
            <TransitionContainer onTransitionEnd={this.onNavigation} {...this.state.transition}>
              <TransitionView viewId={location.key || ''}>
                <Switch key={location.key} location={location}>
                  <Route path={RoutePath.launch} component={Launch} />
                  <Route path={RoutePath.login} component={LoginPage} />
                  <Route path={RoutePath.main} component={MainView} />
                  <Route path={RoutePath.redeemVoucher} component={VoucherInput} />
                  <Route path={RoutePath.voucherSuccess} component={VoucherVerificationSuccess} />
                  <Route path={RoutePath.timeAdded} component={TimeAdded} />
                  <Route path={RoutePath.setupFinished} component={SetupFinished} />
                  <Route path={RoutePath.settings} component={SettingsPage} />
                  <Route path={RoutePath.selectLanguage} component={SelectLanguagePage} />
                  <Route path={RoutePath.accountSettings} component={AccountPage} />
                  <Route path={RoutePath.preferences} component={PreferencesPage} />
                  <Route path={RoutePath.advancedSettings} component={AdvancedSettingsPage} />
                  <Route path={RoutePath.wireguardKeys} component={WireguardKeysPage} />
                  <Route path={RoutePath.splitTunneling} component={SplitTunnelingSettings} />
                  <Route path={RoutePath.support} component={SupportPage} />
                  <Route path={RoutePath.selectLocation} component={SelectLocationPage} />
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

const AppRoutesWithRouter = withHistory(AppRouter);

export default AppRoutesWithRouter;
