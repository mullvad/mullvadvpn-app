import { expect, test } from '@playwright/test';
import { Page } from 'playwright';

import { RoutesObjectModel } from '../../../route-object-models';
import { TestUtils } from '../../../utils';
import { startInstalledApp } from '../../installed-utils';

let page: Page;
let util: TestUtils;
let routes: RoutesObjectModel;

test.describe('Multihop settings', () => {
  const startup = async () => {
    ({ page, util } = await startInstalledApp());
    routes = new RoutesObjectModel(page, util);

    await routes.main.waitForRoute();
    await routes.main.gotoSettings();
    await routes.settings.gotoMultihopSettings();
  };

  test.beforeAll(async () => {
    await startup();
  });

  test.afterAll(async () => {
    await util?.closePage();
  });

  test.afterEach(async () => {
    await routes.multihopSettings.setEnableMultihopSwitch(false);
    const multihopSwitch = routes.multihopSettings.getEnableMultihopSwitch();

    await expect(multihopSwitch).toHaveAttribute('aria-checked', 'false');
  });

  test('Should enable multihop when clicking switch', async () => {
    await routes.multihopSettings.setEnableMultihopSwitch(true);
    const multihopSwitch = routes.multihopSettings.getEnableMultihopSwitch();

    await expect(multihopSwitch).toHaveAttribute('aria-checked', 'true');
  });
});
