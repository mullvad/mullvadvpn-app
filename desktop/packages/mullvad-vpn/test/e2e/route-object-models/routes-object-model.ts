import { Page } from 'playwright';

import { TestUtils } from '../utils';
import { DaitaSettingsRouteObjectModel } from './daita-settings';
import { FilterRouteObjectModel } from './filter';
import { LaunchRouteObjectModel } from './launch';
import { MainRouteObjectModel } from './main';
import { MultihopSettingsRouteObjectModel } from './multihop-settings';
import { SelectLanguageRouteObjectModel } from './select-language';
import { SelectLocationRouteObjectModel } from './select-location';
import { SettingsRouteObjectModel } from './settings/settings-route-object-model';
import { UserInterfaceSettingsRouteObjectModel } from './user-interface-settings';
import { VpnSettingsRouteObjectModel } from './vpn-settings';

export class RoutesObjectModel {
  readonly main: MainRouteObjectModel;
  readonly launch: LaunchRouteObjectModel;
  readonly settings: SettingsRouteObjectModel;
  readonly userInterfaceSettings: UserInterfaceSettingsRouteObjectModel;
  readonly selectLanguage: SelectLanguageRouteObjectModel;
  readonly filter: FilterRouteObjectModel;
  readonly selectLocation: SelectLocationRouteObjectModel;
  readonly vpnSettings: VpnSettingsRouteObjectModel;
  readonly multihopSettings: MultihopSettingsRouteObjectModel;
  readonly daitaSettings: DaitaSettingsRouteObjectModel;

  constructor(page: Page, utils: TestUtils) {
    this.selectLanguage = new SelectLanguageRouteObjectModel(page, utils);
    this.main = new MainRouteObjectModel(page, utils);
    this.launch = new LaunchRouteObjectModel(page, utils);
    this.settings = new SettingsRouteObjectModel(page, utils);
    this.userInterfaceSettings = new UserInterfaceSettingsRouteObjectModel(page, utils);
    this.filter = new FilterRouteObjectModel(page, utils);
    this.selectLocation = new SelectLocationRouteObjectModel(page, utils);
    this.vpnSettings = new VpnSettingsRouteObjectModel(page, utils);
    this.multihopSettings = new MultihopSettingsRouteObjectModel(page, utils);
    this.daitaSettings = new DaitaSettingsRouteObjectModel(page, utils);
  }
}
