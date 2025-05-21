import { Page } from 'playwright';

import { RoutePath } from '../../../../src/renderer/lib/routes';
import { MockedTestUtils } from '../../mocked/mocked-utils';
import { createSelectors } from './selectors';

export class SelectLocationRouteObjectModel {
  private readonly utils: MockedTestUtils;
  private readonly selectors: ReturnType<typeof createSelectors>;

  constructor(page: Page, util: MockedTestUtils) {
    this.utils = util;
    this.selectors = createSelectors(page);
  }

  async toggleAccordion(accordionName: string) {
    const expandAccordion = this.selectors.expandAccordionButton(accordionName);
    if ((await expandAccordion.count()) > 0) {
      await expandAccordion.click();
    }
  }

  getRelaysMatching(relayNames: string[]) {
    return this.selectors.relaysMatching(relayNames);
  }

  getFilterChip(label: string) {
    return this.selectors.filterChip(label);
  }

  async gotoFilter() {
    await this.selectors.filterButton().click();
    await this.utils.waitForRoute(RoutePath.filter);
  }
}
