import { Locator, Page } from 'playwright';

import { RoutePath } from '../../../../src/shared/routes';
import { type TestUtils } from '../../utils';
import { createSelectors } from './selectors';

export class SplitTunnelingSettingsRouteObjectModel {
  readonly page: Page;
  readonly utils: TestUtils;
  public readonly selectors: ReturnType<typeof createSelectors>;

  constructor(page: Page, utils: TestUtils) {
    this.page = page;
    this.utils = utils;
    this.selectors = createSelectors(page);
  }

  async toggleSplitTunneling() {
    await this.selectors.splitTunnelingSwitch().click();
  }

  async toggleApplication(application: Locator) {
    await this.selectors.applicationToogle(application).click();
  }

  async numberOfSplitApplications() {
    const list = this.selectors.splitApplicationsList();

    const listHidden = await list.isHidden();
    if (listHidden) {
      return 0;
    }

    return this.selectors.applicationButtonsInList(list).count();
  }

  getLinuxApplication(applicationName: string) {
    return this.selectors.linuxApplication(applicationName);
  }

  getLinuxApplications() {
    return this.selectors.linuxApplications();
  }

  openFindAnotherApp() {
    return this.selectors.findAnotherAppButton().click();
  }

  openLinuxApplication(applicationName: string) {
    return this.getLinuxApplication(applicationName).click();
  }

  async waitForRoute() {
    await this.utils.expectRoute(RoutePath.splitTunneling);
  }

  // Search input

  async clearSearchInput() {
    await this.selectors.searchInput().clear();
  }

  async fillSearchInput(value: string) {
    await this.selectors.searchInput().fill(value);
  }

  // Launches elsewhere

  closeLinuxApplicationWarningLaunchesElsewhereDialog() {
    return this.selectors.linuxApplicationWarningDialogBackButton().click();
  }

  getLinuxApplicationWarningLaunchesElsewhereDialogText(applicationName: string) {
    return this.selectors.applicationWarningLaunchesElsewhereDialogText(applicationName);
  }

  // Launches in existing process

  closeLinuxApplicationWarningLaunchesInExistingProcessDialog() {
    return this.selectors.linuxApplicationWarningDialogCancelButton().click();
  }

  getLinuxApplicationWarningLaunchesInExistingProcessDialogText(applicationName: string) {
    return this.selectors.applicationWarningLaunchesInExistingProcessDialogText(applicationName);
  }

  openLinuxApplicationFromWarningLaunchesInExistingProcessDialogText() {
    return this.selectors.linuxApplicationWarningDialogLaunchButton().click();
  }

  // Unsupported dialog

  closeUnsupportedDialog() {
    return this.selectors.splitTunnelingUnsupportedDialogCloseButton().click();
  }

  getSplitTunnelingUnsupportedDialogText() {
    return this.selectors.splitTunnelingUnsupportedDialogText();
  }

  openUnsupportedDialog() {
    return this.selectors.splitTunnelingUnsupportedDialogOpenLink().click();
  }
}
