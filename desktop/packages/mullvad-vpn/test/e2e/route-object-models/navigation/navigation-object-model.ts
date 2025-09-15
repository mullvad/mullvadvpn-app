import { Page } from 'playwright';

import { RoutePath } from '../../../../src/shared/routes';
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
    await Promise.all([
      this.utils.waitForNextRoute(),
      this.navigationSelectors.backButton().click(),
    ]);
  }

  async goBackToRoute(route: RoutePath) {
    if (await this.navigationSelectors.backButton().isVisible()) {
      await this.goBack();
      await this.utils.waitForRoute(route);
    }
  }
}
