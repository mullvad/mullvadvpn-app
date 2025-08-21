import { Page } from 'playwright';

import { TestUtils } from '../../utils';
import { NavigationObjectModel } from '../navigation';
import { createSelectors } from './selectors';

export class UdpOverTcpSettingsRouteObjectModel extends NavigationObjectModel {
  readonly selectors: ReturnType<typeof createSelectors>;

  constructor(page: Page, utils: TestUtils) {
    super(page, utils);

    this.selectors = createSelectors(page);
  }

  async selectPort(port: number) {
    await this.selectors.portNumber(port).click();
  }
}
