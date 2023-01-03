import { createRef, useCallback, useEffect, useState } from 'react';
import { Route, Switch } from 'react-router';

import SelectLocation from '../components/select-location/SelectLocationContainer';
import LoginPage from '../containers/LoginPage';
import { useAppContext } from '../context';
import { ITransitionSpecification, transitions, useHistory } from '../lib/history';
import { RoutePath } from '../lib/routes';
import Account from './Account';
import Debug from './Debug';
import { DeviceRevokedView } from './DeviceRevokedView';
import {
  SetupFinished,
  TimeAdded,
  VoucherInput,
  VoucherVerificationSuccess,
} from './ExpiredAccountAddTime';
import Filter from './Filter';
import Focus, { IFocusHandle } from './Focus';
import Launch from './Launch';
import MainView from './MainView';
import OpenVpnSettings from './OpenVpnSettings';
import ProblemReport from './ProblemReport';
import SelectLanguage from './SelectLanguage';
import Settings from './Settings';
import SplitTunnelingSettings from './SplitTunnelingSettings';
import Support from './Support';
import TooManyDevices from './TooManyDevices';
import TransitionContainer, { TransitionView } from './TransitionContainer';
import UserInterfaceSettings from './UserInterfaceSettings';
import VpnSettings from './VpnSettings';
import WireguardSettings from './WireguardSettings';

export default function AppRouter() {
  const history = useHistory();
  const [currentLocation, setCurrentLocation] = useState(history.location);
  const [transition, setTransition] = useState<ITransitionSpecification>(transitions.none);
  const { setNavigationHistory } = useAppContext();
  const focusRef = createRef<IFocusHandle>();

  let unobserveHistory: () => void;

  useEffect(() => {
    // React throttles updates, so it's impossible to capture the intermediate navigation without
    // listening to the history directly.
    unobserveHistory = history.listen((location, _, transition) => {
      setNavigationHistory(history.asObject);
      setCurrentLocation(location);
      setTransition(transition);
    });

    return () => unobserveHistory?.();
  }, []);

  const onNavigation = useCallback(() => {
    focusRef.current?.resetFocus();
  }, []);

  return (
    <Focus ref={focusRef}>
      <TransitionContainer onTransitionEnd={onNavigation} {...transition}>
        <TransitionView viewId={currentLocation.key || ''} routePath={history.location.pathname}>
          <Switch key={currentLocation.key} location={currentLocation}>
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
            <Route exact path={RoutePath.selectLanguage} component={SelectLanguage} />
            <Route exact path={RoutePath.accountSettings} component={Account} />
            <Route exact path={RoutePath.userInterfaceSettings} component={UserInterfaceSettings} />
            <Route exact path={RoutePath.vpnSettings} component={VpnSettings} />
            <Route exact path={RoutePath.wireguardSettings} component={WireguardSettings} />
            <Route exact path={RoutePath.openVpnSettings} component={OpenVpnSettings} />
            <Route exact path={RoutePath.splitTunneling} component={SplitTunnelingSettings} />
            <Route exact path={RoutePath.support} component={Support} />
            <Route exact path={RoutePath.problemReport} component={ProblemReport} />
            <Route exact path={RoutePath.debug} component={Debug} />
            <Route exact path={RoutePath.selectLocation} component={SelectLocation} />
            <Route exact path={RoutePath.filter} component={Filter} />
          </Switch>
        </TransitionView>
      </TransitionContainer>
    </Focus>
  );
}
