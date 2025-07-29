import { Page } from 'playwright';

import { TestUtils } from '../../utils';
import { createSelectors } from './selectors';

export class MultihopSettingsRouteObjectModel {
  readonly page: Page;
  readonly utils: TestUtils;
  readonly selectors: ReturnType<typeof createSelectors>;

  constructor(page: Page, util: TestUtils) {
    this.page = page;
    this.utils = util;
    this.selectors = createSelectors(page);
  }

  getEnableMultihopSwitch() {
    return this.selectors.enableMultihopSwitch();
  }

  async setEnableMultihopSwitch(enabled: boolean) {
    const enableMultihopSwitch = this.selectors.enableMultihopSwitch();
    const ariaChecked = await enableMultihopSwitch.getAttribute('aria-checked');

    if ((enabled && ariaChecked === 'false') || (!enabled && ariaChecked === 'true')) {
      await enableMultihopSwitch.click();
    }
  }
}
