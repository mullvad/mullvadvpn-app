import { Page } from 'playwright';

import { RoutePath } from '../../../../src/shared/routes';
import { generatePath } from '../../lib/path-helpers';
import { TestUtils } from '../../utils';
import { createSelectors } from './selectors';

export class VoucherSuccessRouteObjectModel {
  readonly selectors: ReturnType<typeof createSelectors>;

  constructor(
    private readonly page: Page,
    private readonly utils: TestUtils,
  ) {
    this.selectors = createSelectors(this.page);
  }

  async waitForRoute(newExpiry: string, secondsAdded: number) {
    await this.utils.expectRoute(
      generatePath(RoutePath.voucherSuccess, { newExpiry, secondsAdded: secondsAdded.toString() }),
    );
  }

  async gotoNext() {
    await this.selectors.nextButton().click();
  }
}
