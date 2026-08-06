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
    await routes.multihopSettings.selectMultihopMode('When needed');
  });

  test('Should have selected "When needed" multihop mode by default', async () => {
    const multihopModeItem = routes.multihopSettings.getMultihopModeItem('When needed');
    await expect(multihopModeItem).toHaveAttribute('aria-selected', 'true');
  });

  test('Should select "When needed" from non-default multihop mode', async () => {
    const multihopModeItemNever = routes.multihopSettings.getMultihopModeItem('Never');
    await multihopModeItemNever.click();
    await expect(multihopModeItemNever).toHaveAttribute('aria-selected', 'true');

    const multihopModeItemWhenNeeded = routes.multihopSettings.getMultihopModeItem('When needed');
    await multihopModeItemWhenNeeded.click();
    await expect(multihopModeItemWhenNeeded).toHaveAttribute('aria-selected', 'true');
    await expect(multihopModeItemNever).toHaveAttribute('aria-selected', 'false');
  });

  test('Should select "Always" multihop mode', async () => {
    const multihopModeItem = routes.multihopSettings.getMultihopModeItem('Always');
    await multihopModeItem.click();
    await expect(multihopModeItem).toHaveAttribute('aria-selected', 'true');
  });

  test('Should select "Never" multihop mode', async () => {
    const multihopModeItem = routes.multihopSettings.getMultihopModeItem('Never');
    await multihopModeItem.click();
    await expect(multihopModeItem).toHaveAttribute('aria-selected', 'true');
  });
});
