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

  async setEnableMultihopSwitch(enable: boolean) {
    const enableMultihopSwitch = this.selectors.enableMultihopSwitch();
    const checked = await enableMultihopSwitch.isChecked();

    if ((enable && !checked) || (!enable && checked)) {
      await enableMultihopSwitch.click();
    }
  }
}
