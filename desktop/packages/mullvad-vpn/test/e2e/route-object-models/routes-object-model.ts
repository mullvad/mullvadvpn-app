import { Page } from 'playwright';

import { TestUtils } from '../utils';
import { MainRouteObjectModel } from './main';
import { SelectLanguageRouteObjectModel } from './select-language';
import { SettingsRouteObjectModel } from './settings/settings-route-object-model';
import { UserInterfaceSettingsRouteObjectModel } from './user-interface-settings';

export class RoutesObjectModel {
  readonly main: MainRouteObjectModel;
  readonly settings: SettingsRouteObjectModel;
  readonly userInterfaceSettings: UserInterfaceSettingsRouteObjectModel;
  readonly selectLanguage: SelectLanguageRouteObjectModel;

  constructor(page: Page, utils: TestUtils) {
    this.selectLanguage = new SelectLanguageRouteObjectModel(page, utils);
    this.main = new MainRouteObjectModel(page, utils);
    this.settings = new SettingsRouteObjectModel(page, utils);
    this.userInterfaceSettings = new UserInterfaceSettingsRouteObjectModel(page, utils);
  }
}
