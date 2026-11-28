import { useCallback, useRef } from 'react';
import { Route, Switch } from 'react-router';

import { RoutePath } from '../../shared/routes';
import { useViewTransitions } from '../lib/transition-hooks';
import Focus, { IFocusHandle } from './Focus';
import StateTriggeredNavigation from './StateTriggeredNavigation';
import {
  AntiCensorshipView,
  ApiAccessView,
  AppInfoView,
  AppUpgradeView,
  ChangelogView,
  DaitaSettingsView,
  DebugView,
  EditApiAccessView,
  FilterView,
  ImportConfigView,
  KillSwitchView,
  LaunchView,
  LwoSettingsView,
  MainView,
  MigratedSettingsView,
  MultihopSettingsView,
  ProblemReportView,
  SelectLanguageView,
  SelectLocationView,
  SettingsImportView,
  SettingsTextImportView,
  SettingsView,
  ShadowsocksSettingsView,
  SplitTunnelingView,
  SupportView,
  UdpOverTcpSettingsView,
  UserInterfaceSettingsView,
  VpnSettingsView,
  WireguardConfigView,
  WireguardPortView,
} from './views';

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
          <Route exact path={RoutePath.importConfig} component={ImportConfigView} />
          <Route exact path={RoutePath.main} component={MainView} />
          <Route exact path={RoutePath.migratedSettings} component={MigratedSettingsView} />
          <Route exact path={RoutePath.wireguardConfig} component={WireguardConfigView} />
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
          <Route exact path={RoutePath.apiAccessMethods} component={ApiAccessView} />
          <Route exact path={RoutePath.settingsImport} component={SettingsImportView} />
          <Route exact path={RoutePath.settingsTextImport} component={SettingsTextImportView} />
          <Route exact path={RoutePath.editApiAccessMethods} component={EditApiAccessView} />
          <Route exact path={RoutePath.support} component={SupportView} />
          <Route exact path={RoutePath.problemReport} component={ProblemReportView} />
          <Route exact path={RoutePath.debug} component={DebugView} />
          <Route exact path={RoutePath.selectLocation} component={SelectLocationView} />
          <Route exact path={RoutePath.filter} component={FilterView} />
          <Route exact path={RoutePath.appInfo} component={AppInfoView} />
          <Route exact path={RoutePath.changelog} component={ChangelogView} />
          <Route exact path={RoutePath.appUpgrade} component={AppUpgradeView} />
          <Route exact path={RoutePath.antiCensorship} component={AntiCensorshipView} />
          <Route exact path={RoutePath.wireguardPort} component={WireguardPortView} />
          <Route exact path={RoutePath.lwo} component={LwoSettingsView} />
          <Route exact path={RoutePath.killSwitch} component={KillSwitchView} />
        </Switch>
      </Focus>
    </>
  );
}
