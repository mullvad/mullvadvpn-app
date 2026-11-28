import { Page } from 'playwright';

import { TestUtils } from '../utils';
import { AntiCensorshipRouteObjectModel } from './anti-censorship';
import { DaitaSettingsRouteObjectModel } from './daita-settings';
import { FilterRouteObjectModel } from './filter';
import { ImportConfigRouteObjectModel } from './import-config';
import { LaunchRouteObjectModel } from './launch';
import { LwoSettingsRouteObjectModel } from './lwo-settings';
import { MainRouteObjectModel } from './main';
import { MultihopSettingsRouteObjectModel } from './multihop-settings';
import { SelectLanguageRouteObjectModel } from './select-language';
import { SelectLocationRouteObjectModel } from './select-location';
import { SettingsRouteObjectModel } from './settings/settings-route-object-model';
import { ShadowsocksSettingsRouteObjectModel } from './shadowsocks-settings';
import { SplitTunnelingSettingsRouteObjectModel } from './split-tunneling-settings';
import { UdpOverTcpSettingsRouteObjectModel } from './udp-over-tcp-settings';
import { UserInterfaceSettingsRouteObjectModel } from './user-interface-settings';
import { VpnSettingsRouteObjectModel } from './vpn-settings';
import { WireguardConfigRouteObjectModel } from './wireguard-config';
import { WireGuardPortRouteObjectModel } from './wireguard-port';

export class RoutesObjectModel {
  readonly main: MainRouteObjectModel;
  readonly launch: LaunchRouteObjectModel;
  readonly importConfig: ImportConfigRouteObjectModel;
  readonly wireguardConfig: WireguardConfigRouteObjectModel;
  readonly settings: SettingsRouteObjectModel;
  readonly userInterfaceSettings: UserInterfaceSettingsRouteObjectModel;
  readonly selectLanguage: SelectLanguageRouteObjectModel;
  readonly filter: FilterRouteObjectModel;
  readonly selectLocation: SelectLocationRouteObjectModel;
  readonly vpnSettings: VpnSettingsRouteObjectModel;
  readonly antiCensorship: AntiCensorshipRouteObjectModel;
  readonly udpOverTcpSettings: UdpOverTcpSettingsRouteObjectModel;
  readonly multihopSettings: MultihopSettingsRouteObjectModel;
  readonly daitaSettings: DaitaSettingsRouteObjectModel;
  readonly splitTunnelingSettings: SplitTunnelingSettingsRouteObjectModel;
  readonly shadowsocksSettings: ShadowsocksSettingsRouteObjectModel;
  readonly wireguardPort: WireGuardPortRouteObjectModel;
  readonly lwoSettings: LwoSettingsRouteObjectModel;

  constructor(page: Page, utils: TestUtils) {
    this.selectLanguage = new SelectLanguageRouteObjectModel(page, utils);
    this.main = new MainRouteObjectModel(page, utils);
    this.launch = new LaunchRouteObjectModel(page, utils);
    this.importConfig = new ImportConfigRouteObjectModel(page, utils);
    this.wireguardConfig = new WireguardConfigRouteObjectModel(page, utils);
    this.settings = new SettingsRouteObjectModel(page, utils);
    this.userInterfaceSettings = new UserInterfaceSettingsRouteObjectModel(page, utils);
    this.filter = new FilterRouteObjectModel(page, utils);
    this.selectLocation = new SelectLocationRouteObjectModel(page, utils);
    this.vpnSettings = new VpnSettingsRouteObjectModel(page, utils);
    this.antiCensorship = new AntiCensorshipRouteObjectModel(page, utils);
    this.udpOverTcpSettings = new UdpOverTcpSettingsRouteObjectModel(page, utils);
    this.multihopSettings = new MultihopSettingsRouteObjectModel(page, utils);
    this.daitaSettings = new DaitaSettingsRouteObjectModel(page, utils);
    this.splitTunnelingSettings = new SplitTunnelingSettingsRouteObjectModel(page, utils);
    this.shadowsocksSettings = new ShadowsocksSettingsRouteObjectModel(page, utils);
    this.wireguardPort = new WireGuardPortRouteObjectModel(page, utils);
    this.lwoSettings = new LwoSettingsRouteObjectModel(page, utils);
  }
}
