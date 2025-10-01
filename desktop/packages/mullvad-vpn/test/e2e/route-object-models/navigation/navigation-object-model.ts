import { Page } from 'playwright';

import { RoutePath } from '../../../../src/shared/routes';
import { matchPaths } from '../../lib/path-helpers';
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
    await this.utils.expectRouteChange(() => this.navigationSelectors.backButton().click());
  }

  async goBackToRoute(route: RoutePath) {
    const currentRoute = await this.utils.getCurrentRoute();
    if (!matchPaths(route, currentRoute)) {
      if (await this.navigationSelectors.backButton().isVisible()) {
        await this.goBack();
        await this.goBackToRoute(route);
      } else {
        await this.utils.expectRoute(route);
      }
    }
  }
}
