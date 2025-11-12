import { useCallback, useRef } from 'react';
import { Route, Switch } from 'react-router';

import { RoutePath } from '../../shared/routes';
import SelectLocation from '../components/select-location/SelectLocationContainer';
import { useViewTransitions } from '../lib/transition-hooks';
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
import ProblemReport from './ProblemReport';
import SettingsImport from './SettingsImport';
import SettingsTextImport from './SettingsTextImport';
import StateTriggeredNavigation from './StateTriggeredNavigation';
import {
  Account,
  AntiCensorshipView,
  AppInfoView,
  AppUpgradeView,
  ChangelogView,
  DaitaSettingsView,
  LaunchView,
  LoginView,
  MainView,
  ManageDevicesView,
  MultihopSettingsView,
  SelectLanguageView,
  SettingsView,
  ShadowsocksSettingsView,
  SplitTunnelingView,
  TooManyDevicesView,
  UdpOverTcpSettingsView,
  UserInterfaceSettingsView,
  VpnSettingsView,
  WireguardPortView,
} from './views';
import { SupportView } from './views/support/SupportView';

export default function AppRouter() {
  const focusRef = useRef<IFocusHandle>(null);
  const onNavigation = useCallback(() => {
    focusRef.current?.resetFocus();
  }, [focusRef]);

  const currentLocation = useViewTransitions(onNavigation);

  return (
    <>
      <StateTriggeredNavigation />
      <Focus ref={focusRef}>
        <Switch key={currentLocation.key} location={currentLocation}>
          <Route exact path={RoutePath.launch} component={LaunchView} />
          <Route exact path={RoutePath.login} component={LoginView} />
          <Route exact path={RoutePath.tooManyDevices} component={TooManyDevicesView} />
          <Route exact path={RoutePath.deviceRevoked} component={DeviceRevokedView} />
          <Route exact path={RoutePath.main} component={MainView} />
          <Route exact path={RoutePath.expired} component={ExpiredAccountErrorView} />
          <Route exact path={RoutePath.redeemVoucher} component={VoucherInput} />
          <Route exact path={RoutePath.voucherSuccess} component={VoucherVerificationSuccess} />
          <Route exact path={RoutePath.timeAdded} component={TimeAdded} />
          <Route exact path={RoutePath.setupFinished} component={SetupFinished} />
          <Route exact path={RoutePath.account} component={Account} />
          <Route exact path={RoutePath.settings} component={SettingsView} />
          <Route exact path={RoutePath.selectLanguage} component={SelectLanguageView} />
          <Route
            exact
            path={RoutePath.userInterfaceSettings}
            component={UserInterfaceSettingsView}
          />
          <Route exact path={RoutePath.multihopSettings} component={MultihopSettingsView} />
          <Route exact path={RoutePath.vpnSettings} component={VpnSettingsView} />
          <Route exact path={RoutePath.daitaSettings} component={DaitaSettingsView} />
          <Route exact path={RoutePath.udpOverTcp} component={UdpOverTcpSettingsView} />
          <Route exact path={RoutePath.shadowsocks} component={ShadowsocksSettingsView} />
          <Route exact path={RoutePath.splitTunneling} component={SplitTunnelingView} />
          <Route exact path={RoutePath.apiAccessMethods} component={ApiAccessMethods} />
          <Route exact path={RoutePath.settingsImport} component={SettingsImport} />
          <Route exact path={RoutePath.settingsTextImport} component={SettingsTextImport} />
          <Route exact path={RoutePath.editApiAccessMethods} component={EditApiAccessMethod} />
          <Route exact path={RoutePath.support} component={SupportView} />
          <Route exact path={RoutePath.problemReport} component={ProblemReport} />
          <Route exact path={RoutePath.debug} component={Debug} />
          <Route exact path={RoutePath.selectLocation} component={SelectLocation} />
          <Route exact path={RoutePath.editCustomBridge} component={EditCustomBridge} />
          <Route exact path={RoutePath.filter} component={Filter} />
          <Route exact path={RoutePath.appInfo} component={AppInfoView} />
          <Route exact path={RoutePath.changelog} component={ChangelogView} />
          <Route exact path={RoutePath.appUpgrade} component={AppUpgradeView} />
          <Route exact path={RoutePath.manageDevices} component={ManageDevicesView} />
          <Route exact path={RoutePath.antiCensorship} component={AntiCensorshipView} />
          <Route exact path={RoutePath.wireguardPort} component={WireguardPortView} />
        </Switch>
      </Focus>
    </>
  );
}
