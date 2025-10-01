import { Page } from 'playwright';

import { RoutePath } from '../../../../src/shared/routes';
import { type TestUtils } from '../../utils';
import { createSelectors } from './selectors';

export class SplitTunnelingSettingsRouteObjectModel {
  readonly page: Page;
  readonly utils: TestUtils;
  readonly selectors: ReturnType<typeof createSelectors>;

  constructor(page: Page, utils: TestUtils) {
    this.page = page;
    this.utils = utils;
    this.selectors = createSelectors(page);
  }

  async waitForRoute() {
    await this.utils.expectRoute(RoutePath.splitTunneling);
  }

  async fillSearchInput(value: string) {
    await this.selectors.searchInput().fill(value);
  }

  async clearSearchInput() {
    await this.selectors.searchInput().clear();
  }

  getLinuxApplications() {
    return this.selectors.linuxApplications();
  }

  getSplitTunnelingUnsupportedDialogText() {
    return this.selectors.splitTunnelingUnsupportedDialogText();
  }

  openFindAnotherApp() {
    return this.selectors.findAnotherAppButton().click();
  }

  closeUnsupportedDialog() {
    return this.selectors.splitTunnelingUnsupportedDialogCloseButton().click();
  }

  cancelLinuxApplicationWarningDialog() {
    return this.selectors.linuxApplicationWarningDialogCancelButton().click();
  }

  openLinuxApplicationFromWarningDialog() {
    return this.selectors.linuxApplicationWarningDialogLaunchButton().click();
  }

  getLinuxApplicationWarningDialogText(applicationName: string) {
    return this.selectors.applicationWarningDialogText(applicationName);
  }

  openLinuxApplication(applicationName: string) {
    return this.selectors.linuxApplication(applicationName).click();
  }

  openUnsupportedDialog() {
    return this.selectors.splitTunnelingUnsupportedDialogOpenLink().click();
  }
}
