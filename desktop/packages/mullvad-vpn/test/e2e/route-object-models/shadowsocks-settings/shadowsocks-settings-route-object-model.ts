import { Page } from 'playwright';

import { TestUtils } from '../../utils';
import { NavigationObjectModel } from '../navigation';
import { createSelectors } from './selectors';

export class ShadowsocksSettingsRouteObjectModel extends NavigationObjectModel {
  readonly selectors: ReturnType<typeof createSelectors>;

  constructor(page: Page, utils: TestUtils) {
    super(page, utils);

    this.selectors = createSelectors(page);
  }

  async fillPortInput(port: number) {
    const input = this.selectors.portInput();
    await input.click();
    await input.fill(`${port}`);
    await input.press('Enter');
  }
}
