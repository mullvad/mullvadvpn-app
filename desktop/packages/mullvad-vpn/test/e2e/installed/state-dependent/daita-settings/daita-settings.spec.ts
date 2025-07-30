import { expect, test } from '@playwright/test';
import { Page } from 'playwright';

import { RoutesObjectModel } from '../../../route-object-models';
import { TestUtils } from '../../../utils';
import { startInstalledApp } from '../../installed-utils';

let page: Page;
let util: TestUtils;
let routes: RoutesObjectModel;

test.describe('DAITA settings', () => {
  const startup = async () => {
    ({ page, util } = await startInstalledApp());
    routes = new RoutesObjectModel(page, util);

    await routes.main.waitForRoute();
    await routes.main.gotoSettings();
    await routes.settings.gotoDaitaSettings();
    await routes.daitaSettings.setEnableDaitaSwitch(false);
  };

  test.beforeAll(async () => {
    await startup();
  });

  test.afterAll(async () => {
    await page.close();
  });

  test.afterEach(async () => {
    await routes.daitaSettings.setEnableDaitaSwitch(false);
    const daitaSwitch = routes.daitaSettings.getEnableDaitaSwitch();
    await expect(daitaSwitch).toHaveAttribute('aria-checked', 'true');
  });

  test('Should enable DAITA when clicking switch', async () => {
    const daitaSwitch = routes.daitaSettings.getEnableDaitaSwitch();
    await daitaSwitch.click();
    await expect(daitaSwitch).toHaveAttribute('aria-checked', 'true');
  });
});
