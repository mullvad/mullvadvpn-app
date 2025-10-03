import { Page } from 'playwright';

import { TestUtils } from '../../utils';
import { createSelectors } from './selectors';

export class ManageDevicesRouteObjectModel {
  readonly page: Page;
  readonly utils: TestUtils;
  readonly selectors: ReturnType<typeof createSelectors>;

  constructor(page: Page, util: TestUtils) {
    this.page = page;
    this.utils = util;
    this.selectors = createSelectors(page);
  }
}
