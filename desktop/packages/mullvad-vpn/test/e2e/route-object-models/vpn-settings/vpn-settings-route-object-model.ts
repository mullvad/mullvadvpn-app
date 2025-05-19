import { Page } from 'playwright';

import { TestUtils } from '../../utils';
import { createSelectors } from './selectors';

export class VpnSettingsRouteObjectModel {
  readonly page: Page;
  readonly utils: TestUtils;
  readonly selectors: ReturnType<typeof createSelectors>;

  constructor(page: Page, utils: TestUtils) {
    this.page = page;
    this.utils = utils;
    this.selectors = createSelectors(page);
  }

  getAutoConnectSwitch() {
    return this.selectors.autoConnectSwitch();
  }

  async setAutoConnectSwitch(enabled: boolean) {
    const autoConnectSwitch = this.getAutoConnectSwitch();
    if (enabled) {
      if ((await autoConnectSwitch.getAttribute('aria-checked')) === 'false') {
        await autoConnectSwitch.click();
      }
    } else {
      if ((await autoConnectSwitch.getAttribute('aria-checked')) === 'true') {
        await autoConnectSwitch.click();
      }
    }
    return autoConnectSwitch;
  }

  getLaunchAppOnStartupSwitch() {
    return this.selectors.launchAppOnStartupSwitch();
  }

  async setLaunchAppOnStartupSwitch(enabled: boolean) {
    const launchAppOnStartupSwitch = this.getLaunchAppOnStartupSwitch();
    if (enabled) {
      if ((await launchAppOnStartupSwitch.getAttribute('aria-checked')) === 'false') {
        await launchAppOnStartupSwitch.click();
      }
    } else {
      if ((await launchAppOnStartupSwitch.getAttribute('aria-checked')) === 'true') {
        await launchAppOnStartupSwitch.click();
      }
    }
    return launchAppOnStartupSwitch;
  }

  getLanSwitch() {
    return this.selectors.lanSwitch();
  }

  async setLanSwitch(enabled: boolean) {
    const lanSwitch = this.getLanSwitch();
    if (enabled) {
      if ((await lanSwitch.getAttribute('aria-checked')) === 'false') {
        await lanSwitch.click();
      }
    } else {
      if ((await lanSwitch.getAttribute('aria-checked')) === 'true') {
        await lanSwitch.click();
      }
    }
    return lanSwitch;
  }
}
