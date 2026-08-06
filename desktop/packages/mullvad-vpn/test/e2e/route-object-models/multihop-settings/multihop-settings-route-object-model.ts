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

  getMultihopModeItems() {
    return this.selectors.multihopModeItems();
  }

  getMultihopModeItem(label: string) {
    return this.selectors.multihopModeItem(this.getMultihopModeItems(), label);
  }

  async selectMultihopMode(label: string) {
    const multihopModeItem = this.getMultihopModeItem(label);
    await multihopModeItem.click();
  }
}
