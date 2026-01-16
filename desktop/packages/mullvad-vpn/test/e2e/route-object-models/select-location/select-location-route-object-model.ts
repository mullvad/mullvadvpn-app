import { Page } from 'playwright';

import { RoutePath } from '../../../../src/shared/routes';
import { TestUtils } from '../../utils';
import { createSelectors } from './selectors';

export class SelectLocationRouteObjectModel {
  public readonly selectors: ReturnType<typeof createSelectors>;
  private readonly utils: TestUtils;

  constructor(page: Page, util: TestUtils) {
    this.utils = util;
    this.selectors = createSelectors(page);
  }

  async toggleAccordion(accordionName: string) {
    const expandAccordion = this.selectors.expandAccordionButton(accordionName);
    if ((await expandAccordion.count()) > 0) {
      await expandAccordion.click();
    }
  }

  async gotoFilter() {
    await this.selectors.filterButton().click();
    await this.utils.expectRoute(RoutePath.filter);
  }
}
