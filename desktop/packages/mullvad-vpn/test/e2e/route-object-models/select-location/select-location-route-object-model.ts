import { Page } from 'playwright';

import { RoutePath } from '../../../../src/shared/routes';
import { TestUtils } from '../../utils';
import { createSelectors } from './selectors';

export class SelectLocationRouteObjectModel {
  private readonly utils: TestUtils;
  private readonly selectors: ReturnType<typeof createSelectors>;

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

  getEntryButton() {
    return this.selectors.entryButton();
  }

  getExitButton() {
    return this.selectors.exitButton();
  }

  getSearchInput() {
    return this.selectors.searchInput();
  }

  getRelaysMatching(relayNames: string[]) {
    return this.selectors.relaysMatching(relayNames);
  }

  getFilterChip(label: string) {
    return this.selectors.filterChip(label);
  }

  async gotoFilter() {
    await this.selectors.filterButton().click();
    await this.utils.expectRoute(RoutePath.filter);
  }
}
