import { Page } from 'playwright';

import { RoutePath } from '../../../../src/shared/routes';
import { TestUtils } from '../../utils';
import { createSelectors } from './selectors';

export class ExpiredRouteObjectModel {
  readonly selectors: ReturnType<typeof createSelectors>;

  constructor(
    private readonly page: Page,
    private readonly utils: TestUtils,
  ) {
    this.selectors = createSelectors(this.page);
  }

  async waitForRoute() {
    await this.utils.expectRoute(RoutePath.expired);
  }

  async gotoRedeemVoucher() {
    await this.selectors.redeemVoucherButton().click();
    await this.utils.expectRoute(RoutePath.redeemVoucher);
  }
}
