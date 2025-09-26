import { Page } from 'playwright';

import { TestUtils } from '../../utils';
import { createSelectors } from './selectors';

export class NavigationObjectModel {
  readonly navigationSelectors: ReturnType<typeof createSelectors>;

  constructor(
    protected readonly page: Page,
    protected readonly utils: TestUtils,
  ) {
    this.navigationSelectors = createSelectors(page);
  }

  async goBack(levels = 1) {
    for (let i = 0; i < levels; i++) {
      await Promise.all([
        this.utils.waitForNextRoute(),
        this.navigationSelectors.backButton().click(),
      ]);
    }
  }
}
