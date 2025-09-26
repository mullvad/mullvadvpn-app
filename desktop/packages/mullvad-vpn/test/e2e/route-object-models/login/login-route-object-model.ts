import { Page } from 'playwright';

import { RoutePath } from '../../../../src/shared/routes';
import { TestUtils } from '../../utils';
import { createSelectors } from './selectors';

export class LoginRouteObjectModel {
  readonly selectors: ReturnType<typeof createSelectors>;

  constructor(
    private readonly page: Page,
    private readonly utils: TestUtils,
  ) {
    this.selectors = createSelectors(this.page);
  }

  async waitForRoute() {
    await this.utils.expectRoute(RoutePath.login);
  }

  fillAccountNumber(accountNumber: string) {
    return this.selectors.loginInput().fill(accountNumber);
  }

  async loginByPressingEnter() {
    await this.selectors.loginInput().press('Enter');
  }

  async loginByClickingLoginButton() {
    await this.selectors.loginButton().click();
  }

  async createNewAccount() {
    await this.selectors.createNewAccountButton().click();
  }

  getCreateNewAccountConfirmationMessage() {
    return this.selectors.createNewAccountMessage();
  }

  async confirmCreateNewAccount() {
    await this.selectors.confirmCreateNewAccountButton().click();
  }

  async cancelCreateNewAccount() {
    await this.selectors.cancelDialogButton().click();
  }

  async clearAccountHistory() {
    await this.selectors.clearAccountHistory().click();
  }

  getAccountHistoryItemButton() {
    return this.selectors.accountHistoryItemButton();
  }

  getClearAccountHistoryConfirmationMessage() {
    return this.selectors.clearAccountHistoryMessage();
  }

  async confirmClearAccountHistory() {
    await this.selectors.confirmClearAccountHistoryButton().click();
  }

  async cancelClearAccountHistory() {
    await this.selectors.cancelDialogButton().click();
  }
}
