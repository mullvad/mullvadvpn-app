import { Action } from 'history';
import * as React from 'react';
import { Route, Switch } from 'react-router';

import AccountPage from '../containers/AccountPage';
import LoginPage from '../containers/LoginPage';
import ProblemReportPage from '../containers/ProblemReportPage';
import SelectLanguagePage from '../containers/SelectLanguagePage';
import SelectLocationPage from '../containers/SelectLocationPage';
import withAppContext, { IAppContext } from '../context';
import { IHistoryProps, ITransitionSpecification, transitions, withHistory } from '../lib/history';
import { RoutePath } from '../lib/routes';
import { DeviceRevokedView } from './DeviceRevokedView';
import {
  SetupFinished,
  TimeAdded,
  VoucherInput,
  VoucherVerificationSuccess,
} from './ExpiredAccountAddTime';
import Filter from './Filter';
import Focus, { IFocusHandle } from './Focus';
import InterfaceSettings from './InterfaceSettings';
import Launch from './Launch';
import MainView from './MainView';
import OpenVpnSettings from './OpenVpnSettings';
import Settings from './Settings';
import SplitTunnelingSettings from './SplitTunnelingSettings';
import Support from './Support';
import TooManyDevices from './TooManyDevices';
import TransitionContainer, { TransitionView } from './TransitionContainer';
import VpnSettings from './VpnSettings';
import WireguardSettings from './WireguardSettings';

interface IAppRoutesState {
  currentLocation: IHistoryProps['history']['location'];
  transition: ITransitionSpecification;
  action?: Action;
}

class AppRouter extends React.Component<IHistoryProps & IAppContext, IAppRoutesState> {
  private unobserveHistory?: () => void;

  private focusRef = React.createRef<IFocusHandle>();

  constructor(props: IHistoryProps & IAppContext) {
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
      this.props.app.setNavigationHistory(this.props.history.asObject);
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
      <Focus ref={this.focusRef}>
        <TransitionContainer onTransitionEnd={this.onNavigation} {...this.state.transition}>
          <TransitionView viewId={location.key || ''}>
            <Switch key={location.key} location={location}>
              <Route exact path={RoutePath.launch} component={Launch} />
              <Route exact path={RoutePath.login} component={LoginPage} />
              <Route exact path={RoutePath.tooManyDevices} component={TooManyDevices} />
              <Route exact path={RoutePath.deviceRevoked} component={DeviceRevokedView} />
              <Route exact path={RoutePath.main} component={MainView} />
              <Route exact path={RoutePath.redeemVoucher} component={VoucherInput} />
              <Route exact path={RoutePath.voucherSuccess} component={VoucherVerificationSuccess} />
              <Route exact path={RoutePath.timeAdded} component={TimeAdded} />
              <Route exact path={RoutePath.setupFinished} component={SetupFinished} />
              <Route exact path={RoutePath.settings} component={Settings} />
              <Route exact path={RoutePath.selectLanguage} component={SelectLanguagePage} />
              <Route exact path={RoutePath.accountSettings} component={AccountPage} />
              <Route exact path={RoutePath.interfaceSettings} component={InterfaceSettings} />
              <Route exact path={RoutePath.vpnSettings} component={VpnSettings} />
              <Route exact path={RoutePath.wireguardSettings} component={WireguardSettings} />
              <Route exact path={RoutePath.openVpnSettings} component={OpenVpnSettings} />
              <Route exact path={RoutePath.splitTunneling} component={SplitTunnelingSettings} />
              <Route exact path={RoutePath.support} component={Support} />
              <Route exact path={RoutePath.problemReport} component={ProblemReportPage} />
              <Route exact path={RoutePath.selectLocation} component={SelectLocationPage} />
              <Route exact path={RoutePath.filter} component={Filter} />
            </Switch>
          </TransitionView>
        </TransitionContainer>
      </Focus>
    );
  }

  private onNavigation = () => {
    this.focusRef.current?.resetFocus();
  };
}

const AppRoutesWithRouter = withAppContext(withHistory(AppRouter));

export default AppRoutesWithRouter;
