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
    await this.utils.waitForRoute(RoutePath.splitTunneling);
  }

  getLinuxApplications() {
    return this.selectors.linuxApplications();
  }

  getSplitTunnelingUnsupportedDialogText() {
    return this.selectors.splitTunnelingUnsupportedDialogText();
  }

  closeUnsupportedDialog() {
    return this.selectors.splitTunnelingUnsupportedDialogCloseButton().click();
  }

  openUnsupportedDialog() {
    return this.selectors.splitTunnelingUnsupportedDialogOpenLink().click();
  }
}
