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
    await this.utils.waitForRoute(RoutePath.redeemVoucher);
  }

  async fillVoucherInput() {
    await this.selectors.voucherInput().fill('1234-5678-90AB-CDEF');
  }

  async redeemVoucher() {
    await this.selectors.redeemButton().click();
  }
}
