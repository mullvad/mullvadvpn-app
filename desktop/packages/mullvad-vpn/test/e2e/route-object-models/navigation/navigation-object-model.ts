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

  async goBack() {
    await this.utils.waitForRouteChange(() => this.navigationSelectors.backButton().click());
  }

  async gotoRoot() {
    await this.page.press('body', 'Shift+Escape');
  }
}
