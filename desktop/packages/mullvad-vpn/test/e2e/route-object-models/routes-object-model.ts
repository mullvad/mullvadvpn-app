import { Page } from 'playwright';

import { TestUtils } from '../utils';
import { DaitaSettingsRouteObjectModel } from './daita-settings';
import { DeviceRevokedRouteObjectModel } from './device-revoked';
import { ExpiredRouteObjectModel } from './expired';
import { FilterRouteObjectModel } from './filter';
import { LaunchRouteObjectModel } from './launch';
import { LoginRouteObjectModel } from './login';
import { MainRouteObjectModel } from './main';
import { MultihopSettingsRouteObjectModel } from './multihop-settings';
import { RedeemVoucherRouteObjectModel } from './redeem-voucher';
import { SelectLanguageRouteObjectModel } from './select-language';
import { SelectLocationRouteObjectModel } from './select-location';
import { SettingsRouteObjectModel } from './settings/settings-route-object-model';
import { SetupFinishedRouteObjectModel } from './setup-finished';
import { SplitTunnelingSettingsRouteObjectModel } from './split-tunneling-settings';
import { TimeAddedRouteObjectModel } from './time-added';
import { TooManyDevicesRouteObjectModel } from './too-many-devices';
import { UdpOverTcpSettingsRouteObjectModel } from './udp-over-tcp-settings';
import { UserInterfaceSettingsRouteObjectModel } from './user-interface-settings';
import { VoucherSuccessRouteObjectModel } from './voucher-success';
import { VpnSettingsRouteObjectModel } from './vpn-settings';
import { WireguardSettingsRouteObjectModel } from './wireguard-settings';

export class RoutesObjectModel {
  readonly main: MainRouteObjectModel;
  readonly launch: LaunchRouteObjectModel;
  readonly login: LoginRouteObjectModel;
  readonly expired: ExpiredRouteObjectModel;
  readonly redeemVoucher: RedeemVoucherRouteObjectModel;
  readonly voucherSuccess: VoucherSuccessRouteObjectModel;
  readonly timeAdded: TimeAddedRouteObjectModel;
  readonly setupFinished: SetupFinishedRouteObjectModel;
  readonly deviceRevoked: DeviceRevokedRouteObjectModel;
  readonly tooManyDevices: TooManyDevicesRouteObjectModel;
  readonly settings: SettingsRouteObjectModel;
  readonly userInterfaceSettings: UserInterfaceSettingsRouteObjectModel;
  readonly selectLanguage: SelectLanguageRouteObjectModel;
  readonly filter: FilterRouteObjectModel;
  readonly selectLocation: SelectLocationRouteObjectModel;
  readonly vpnSettings: VpnSettingsRouteObjectModel;
  readonly wireguardSettings: WireguardSettingsRouteObjectModel;
  readonly udpOverTcpSettings: UdpOverTcpSettingsRouteObjectModel;
  readonly multihopSettings: MultihopSettingsRouteObjectModel;
  readonly daitaSettings: DaitaSettingsRouteObjectModel;
  readonly splitTunnelingSettings: SplitTunnelingSettingsRouteObjectModel;

  constructor(page: Page, utils: TestUtils) {
    this.selectLanguage = new SelectLanguageRouteObjectModel(page, utils);
    this.main = new MainRouteObjectModel(page, utils);
    this.launch = new LaunchRouteObjectModel(page, utils);
    this.login = new LoginRouteObjectModel(page, utils);
    this.expired = new ExpiredRouteObjectModel(page, utils);
    this.redeemVoucher = new RedeemVoucherRouteObjectModel(page, utils);
    this.voucherSuccess = new VoucherSuccessRouteObjectModel(page, utils);
    this.timeAdded = new TimeAddedRouteObjectModel(page, utils);
    this.setupFinished = new SetupFinishedRouteObjectModel(page, utils);
    this.deviceRevoked = new DeviceRevokedRouteObjectModel(page, utils);
    this.tooManyDevices = new TooManyDevicesRouteObjectModel(page, utils);
    this.settings = new SettingsRouteObjectModel(page, utils);
    this.userInterfaceSettings = new UserInterfaceSettingsRouteObjectModel(page, utils);
    this.filter = new FilterRouteObjectModel(page, utils);
    this.selectLocation = new SelectLocationRouteObjectModel(page, utils);
    this.vpnSettings = new VpnSettingsRouteObjectModel(page, utils);
    this.wireguardSettings = new WireguardSettingsRouteObjectModel(page, utils);
    this.udpOverTcpSettings = new UdpOverTcpSettingsRouteObjectModel(page, utils);
    this.multihopSettings = new MultihopSettingsRouteObjectModel(page, utils);
    this.daitaSettings = new DaitaSettingsRouteObjectModel(page, utils);
    this.splitTunnelingSettings = new SplitTunnelingSettingsRouteObjectModel(page, utils);
  }
}
