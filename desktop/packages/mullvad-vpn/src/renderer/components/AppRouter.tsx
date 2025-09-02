import { useCallback, useRef } from 'react';
import { Route, Switch } from 'react-router';

import { RoutePath } from '../../shared/routes';
import SelectLocation from '../components/select-location/SelectLocationContainer';
import { useViewTransitions } from '../lib/transition-hooks';
import Account from './Account';
import ApiAccessMethods from './ApiAccessMethods';
import Debug from './Debug';
import { DeviceRevokedView } from './DeviceRevokedView';
import { EditApiAccessMethod } from './EditApiAccessMethod';
import { EditCustomBridge } from './EditCustomBridge';
import {
  SetupFinished,
  TimeAdded,
  VoucherInput,
  VoucherVerificationSuccess,
} from './ExpiredAccountAddTime';
import ExpiredAccountErrorView from './ExpiredAccountErrorView';
import Filter from './Filter';
import Focus, { IFocusHandle } from './Focus';
import OpenVpnSettings from './OpenVpnSettings';
import ProblemReport from './ProblemReport';
import SelectLanguage from './SelectLanguage';
import SettingsImport from './SettingsImport';
import SettingsTextImport from './SettingsTextImport';
import Support from './Support';
import TooManyDevices from './TooManyDevices';
import UserInterfaceSettings from './UserInterfaceSettings';
import {
  AppInfoView,
  AppUpgradeView,
  ChangelogView,
  DaitaSettingsView,
  LaunchView,
  LoginView,
  MainView,
  MultihopSettingsView,
  SettingsView,
  ShadowsocksSettingsView,
  SplitTunnelingSettingsView,
  UdpOverTcp,
  VpnSettingsView,
  WireguardSettingsView,
} from './views';

export default function AppRouter() {
  const focusRef = useRef<IFocusHandle>(null);
  const onNavigation = useCallback(() => {
    focusRef.current?.resetFocus();
  }, [focusRef]);

  const currentLocation = useViewTransitions(onNavigation);

  return (
    <Focus ref={focusRef}>
      <Switch key={currentLocation.key} location={currentLocation}>
        <Route exact path={RoutePath.launch} component={LaunchView} />
        <Route exact path={RoutePath.login} component={LoginView} />
        <Route exact path={RoutePath.tooManyDevices} component={TooManyDevices} />
        <Route exact path={RoutePath.deviceRevoked} component={DeviceRevokedView} />
        <Route exact path={RoutePath.main} component={MainView} />
        <Route exact path={RoutePath.expired} component={ExpiredAccountErrorView} />
        <Route exact path={RoutePath.redeemVoucher} component={VoucherInput} />
        <Route exact path={RoutePath.voucherSuccess} component={VoucherVerificationSuccess} />
        <Route exact path={RoutePath.timeAdded} component={TimeAdded} />
        <Route exact path={RoutePath.setupFinished} component={SetupFinished} />
        <Route exact path={RoutePath.account} component={Account} />
        <Route exact path={RoutePath.settings} component={SettingsView} />
        <Route exact path={RoutePath.selectLanguage} component={SelectLanguage} />
        <Route exact path={RoutePath.userInterfaceSettings} component={UserInterfaceSettings} />
        <Route exact path={RoutePath.multihopSettings} component={MultihopSettingsView} />
        <Route exact path={RoutePath.vpnSettings} component={VpnSettingsView} />
        <Route exact path={RoutePath.wireguardSettings} component={WireguardSettingsView} />
        <Route exact path={RoutePath.daitaSettings} component={DaitaSettingsView} />
        <Route exact path={RoutePath.udpOverTcp} component={UdpOverTcp} />
        <Route exact path={RoutePath.shadowsocks} component={ShadowsocksSettingsView} />
        <Route exact path={RoutePath.openVpnSettings} component={OpenVpnSettings} />
        <Route exact path={RoutePath.splitTunneling} component={SplitTunnelingSettingsView} />
        <Route exact path={RoutePath.apiAccessMethods} component={ApiAccessMethods} />
        <Route exact path={RoutePath.settingsImport} component={SettingsImport} />
        <Route exact path={RoutePath.settingsTextImport} component={SettingsTextImport} />
        <Route exact path={RoutePath.editApiAccessMethods} component={EditApiAccessMethod} />
        <Route exact path={RoutePath.support} component={Support} />
        <Route exact path={RoutePath.problemReport} component={ProblemReport} />
        <Route exact path={RoutePath.debug} component={Debug} />
        <Route exact path={RoutePath.selectLocation} component={SelectLocation} />
        <Route exact path={RoutePath.editCustomBridge} component={EditCustomBridge} />
        <Route exact path={RoutePath.filter} component={Filter} />
        <Route exact path={RoutePath.appInfo} component={AppInfoView} />
        <Route exact path={RoutePath.changelog} component={ChangelogView} />
        <Route exact path={RoutePath.appUpgrade} component={AppUpgradeView} />
      </Switch>
    </Focus>
  );
}
