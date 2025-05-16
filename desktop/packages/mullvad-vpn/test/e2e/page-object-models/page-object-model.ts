import { Page } from 'playwright';

import { MockedTestUtils } from '../mocked/mocked-utils';
import { MainPom } from './main';
import { SelectLanguagePom } from './select-language';
import { SettingsPom } from './settings/settings-pom';
import { UserInterfaceSettingsPom } from './user-interface-settings';

export class PageObjectModel {
  readonly main: MainPom;
  readonly settings: SettingsPom;
  readonly userInterfaceSettings: UserInterfaceSettingsPom;
  readonly selectLanguage: SelectLanguagePom;

  constructor(page: Page, utils: MockedTestUtils) {
    this.selectLanguage = new SelectLanguagePom(page, utils);
    this.main = new MainPom(page, utils);
    this.settings = new SettingsPom(page, utils);
    this.userInterfaceSettings = new UserInterfaceSettingsPom(page, utils);
  }
}
