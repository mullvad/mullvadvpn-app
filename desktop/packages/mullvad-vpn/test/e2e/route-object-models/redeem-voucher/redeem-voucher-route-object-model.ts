import { Page } from 'playwright';

import { RoutePath } from '../../../../src/shared/routes';
import { TestUtils } from '../../utils';
import { createSelectors } from './selectors';

export class RedeemVoucherRouteObjectModel {
  readonly selectors: ReturnType<typeof createSelectors>;

  constructor(
    private readonly page: Page,
    private readonly utils: TestUtils,
  ) {
    this.selectors = createSelectors(this.page);
  }

  async waitForRoute() {
    await this.utils.expectRoute(RoutePath.redeemVoucher);
  }

  async fillVoucherInput(accountNumber: string) {
    await this.selectors.voucherInput().fill(accountNumber);
  }

  async redeemVoucher() {
    await this.selectors.redeemButton().click();
  }
}
