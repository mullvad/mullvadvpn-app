import { Page } from 'playwright';

import { TestUtils } from '../../utils';
import { createSelectors } from './selectors';

export class DaitaSettingsRouteObjectModel {
  readonly page: Page;
  readonly utils: TestUtils;
  readonly selectors: ReturnType<typeof createSelectors>;

  constructor(page: Page, utils: TestUtils) {
    this.page = page;
    this.utils = utils;
    this.selectors = createSelectors(page);
  }

  getEnableDaitaSwitch() {
    return this.selectors.enableDaitaSwitch();
  }

  async setEnableDaitaSwitch(enable: boolean) {
    const enableDaitaSwitch = this.selectors.enableDaitaSwitch();
    const checked = await enableDaitaSwitch.isChecked();

    if ((enable && !checked) || (!enable && checked)) {
      await enableDaitaSwitch.click();
    }
  }
}
