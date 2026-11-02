import { type Locator, Page } from 'playwright';

import { RoutePath } from '../../../../src/shared/routes';
import { type MockLocationType } from '../../utils';
import { TestUtils } from '../../utils';
import { createSelectors } from './selectors';

export class SelectLocationRouteObjectModel {
  private readonly utils: TestUtils;
  private readonly selectors: ReturnType<typeof createSelectors>;
  private readonly page: Page;

  constructor(page: Page, util: TestUtils) {
    this.utils = util;
    this.page = page;
    this.selectors = createSelectors(page);
  }

  async expandAccordion(accordionName: string) {
    const accordionLocator = this.selectors.accordionButton(accordionName);
    const label = await accordionLocator.getAttribute('aria-label');
    if (label?.startsWith('Expand')) {
      await accordionLocator.click();
    }
  }

  async gotoEntryLocations() {
    await this.getEntryInput().click();
    await this.page.waitForFunction(() =>
      document.getAnimations().every((animation) => animation.playState === 'finished'),
    );
  }

  async gotoEntryAutomatic(country: string) {
    await this.getEntryAutomaticButton(country).click();
    await this.page.waitForFunction(() =>
      document.getAnimations().every((animation) => animation.playState === 'finished'),
    );
  }

  async gotoExitLocations() {
    await this.getExitInput().click();
    await this.page.waitForFunction(() =>
      document.getAnimations().every((animation) => animation.playState === 'finished'),
    );
  }

  async gotoLocations(locationType: MockLocationType) {
    if (locationType === 'exit') {
      await this.gotoExitLocations();
    } else {
      await this.gotoEntryLocations();
    }
  }

  getEntryInput() {
    return this.selectors.entryInput();
  }

  getEntryAutomaticButton(country: string) {
    return this.selectors.entryAutomaticButton(country);
  }

  getExitInput() {
    return this.selectors.exitInput();
  }

  getNonMatchingFilterErrorMessage() {
    return this.selectors.nonMatchingFilterErrorMessage();
  }

  getLocationsMatching(relayNames: string[]) {
    return this.selectors.locationsMatching(relayNames);
  }

  getAutomaticLocation() {
    return this.selectors.automaticLocation();
  }

  getAutomaticEntryMultihopModeToAlwaysButton() {
    return this.selectors.entryAutomaticMultihopModeToAlwaysButton();
  }

  getFilterChipButton(label: string) {
    return this.selectors.filterChipButton(label);
  }

  getFilterChipLink(label: string) {
    return this.selectors.filterChipLink(label);
  }

  getAllLocationsSection() {
    return this.selectors.allLocationsSection();
  }

  getRecentsSection() {
    return this.selectors.recentSection();
  }

  getLocationsInLocator(locator: Locator) {
    return this.selectors.locations(locator);
  }

  getLocationsInAllLocations() {
    const allLocationsSection = this.getAllLocationsSection();
    return this.getLocationsInLocator(allLocationsSection);
  }

  getLocationsInRecents() {
    const recentsSection = this.selectors.recentSection();
    return this.selectors.locations(recentsSection);
  }

  getRecentMenuButton(locationName: string) {
    const recentsSection = this.selectors.recentSection();
    return this.selectors.locationMenuButton(locationName, recentsSection);
  }

  getAddToCustomListButton(locationName: string, customListName: string) {
    return this.selectors.addToCustomListButton(locationName, customListName);
  }

  getAddToNewCustomListButton(locationName: string) {
    return this.selectors.addToNewCustomListButton(locationName);
  }

  getEditCustomListButton() {
    return this.selectors.editCustomListButton();
  }

  getDeleteCustomListButton() {
    return this.selectors.deleteCustomListButton();
  }

  getAutomaticLocationWarningDialogContinueButton() {
    return this.selectors.automaticLocationWarningDialogContinueButton();
  }

  async gotoEntryFilter() {
    await this.selectors.filterEntryButton().click();
    await this.utils.expectRoute(RoutePath.filter);
  }

  async gotoEntryAutomaticFilter() {
    await this.selectors.filterEntryAutomaticButton().click();
    await this.utils.expectRoute(RoutePath.filter);
  }

  async gotoExitFilter() {
    await this.selectors.filterExitButton().click();
    await this.utils.expectRoute(RoutePath.filter);
  }
}
