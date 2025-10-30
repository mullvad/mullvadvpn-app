import { Page } from 'playwright';

import { TestUtils } from '../../utils';
import { NavigationObjectModel } from '../navigation';
import { createSelectors } from './selectors';

export class WireGuardPortRouteObjectModel extends NavigationObjectModel {
  readonly selectors: ReturnType<typeof createSelectors>;

  constructor(
    public readonly page: Page,
    public readonly utils: TestUtils,
  ) {
    super(page, utils);
    this.selectors = createSelectors(page);
  }
}
