import { Page } from 'playwright';

import { RoutePath } from '../../../../src/renderer/lib/routes';
import { MockedTestUtils } from '../../mocked/mocked-utils';
import { createSelectors, FilterSelectors } from './selectors';

export class FilterRouteObjectModel {
  private readonly utils: MockedTestUtils;
  private readonly selectors: FilterSelectors;

  constructor(page: Page, util: MockedTestUtils) {
    this.utils = util;
    this.selectors = createSelectors(page);
  }

  async applyFilter() {
    await this.selectors.applyButton().click();
  }

  async gotoSelectLocation() {
    await this.selectors.backButton().click();
    await this.utils.waitForRoute(RoutePath.selectLocation);
  }

  async expandProviders() {
    const accordion = this.selectors.accordionContainer('provider-accordion-container');
    if ((await accordion.evaluate((el) => getComputedStyle(el).height)) === '0px') {
      await this.selectors.accordion('Providers').click();
    }
  }

  async collapseProviders() {
    const accordion = this.selectors.accordionContainer('provider-accordion-container');
    if ((await accordion.evaluate((el) => getComputedStyle(el).height)) !== '0px') {
      await this.selectors.accordion('Providers').click();
    }
  }

  async checkAllProvidersCheckbox() {
    const allProvidersCheckbox = this.selectors.providersOption('All providers');
    await allProvidersCheckbox.click();
  }

  async checkProviderCheckbox(provider: string) {
    const providerCheckbox = this.selectors.providersOption(provider);
    await providerCheckbox.click();
  }

  async expandOwnership() {
    const accordion = this.selectors.accordionContainer('ownership-accordion-container');
    const accordionContainerHeight = await accordion.evaluate((el) => getComputedStyle(el).height);
    if (accordionContainerHeight === '0px') {
      await this.selectors.accordion('Ownership').click();
    }
  }

  async collapseOwnership() {
    const accordion = this.selectors.accordionContainer('ownership-accordion-container');
    const accordionContainerHeight = await accordion.evaluate((el) => getComputedStyle(el).height);
    if (accordionContainerHeight !== '0px') {
      await this.selectors.accordion('Ownership').click();
    }
  }

  async selectOwnershipOption(ownership: string) {
    await this.selectors.ownershipOption(ownership).click();
  }
}
