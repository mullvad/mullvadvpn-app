import { Locator, Page } from 'playwright';

import { RoutePath } from '../../../../src/shared/routes';
import { TestUtils } from '../../utils';
import { createSelectors, FilterSelectors } from './selectors';

export class FilterRouteObjectModel {
  private readonly utils: TestUtils;
  private readonly selectors: FilterSelectors;

  constructor(page: Page, util: TestUtils) {
    this.utils = util;
    this.selectors = createSelectors(page);
  }

  async applyFilter() {
    await this.selectors.applyButton().click();
  }

  async gotoSelectLocation() {
    await this.selectors.backButton().click();
    await this.utils.expectRoute(RoutePath.selectLocation);
  }

  async expandProviders() {
    const accordion = this.selectors.accordion('Providers');
    const expanded = await this.isExpanded(accordion);
    if (!expanded) {
      await accordion.click();
      await this.selectors.providersOption('All providers').waitFor({ state: 'visible' });
    }
  }

  async collapseProviders() {
    const accordion = this.selectors.accordion('Providers');
    const expanded = await this.isExpanded(accordion);
    if (expanded) await accordion.click();
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
    const accordion = this.selectors.accordion('Ownership');
    const expanded = await this.isExpanded(accordion);
    if (!expanded) {
      await accordion.click();
      await this.selectors.ownershipOption('Any').waitFor({ state: 'visible' });
    }
  }

  async collapseOwnership() {
    const accordion = this.selectors.accordion('Ownership');
    const expanded = await this.isExpanded(accordion);
    if (expanded) await accordion.click();
  }

  async selectOwnershipOption(ownership: string) {
    await this.selectors.ownershipOption(ownership).click();
  }

  private async isExpanded(locator: Locator): Promise<boolean> {
    const ariaExpanded = await locator.getAttribute('aria-expanded');
    return ariaExpanded === 'true';
  }
}
