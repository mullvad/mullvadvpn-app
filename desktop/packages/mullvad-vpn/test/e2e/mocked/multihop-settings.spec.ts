import { expect, test } from '@playwright/test';
import { Page } from 'playwright';

import { getDefaultSettings } from '../../../src/main/default-settings';
import { mockData } from '../mock-data';
import { RoutesObjectModel } from '../route-object-models';
import { MockedTestUtils, startMockedApp } from './mocked-utils';

const { relayList } = mockData;

let page: Page;
let util: MockedTestUtils;
let routes: RoutesObjectModel;

test.describe('Multihop settings', () => {
  test.beforeAll(async () => {
    await startup();
  });

  test.afterAll(async () => {
    await util?.closePage();
  });

  const startup = async () => {
    ({ page, util } = await startMockedApp());
    routes = new RoutesObjectModel(page, util);

    await routes.main.waitForRoute();
    await routes.main.gotoSettings();
    await routes.settings.gotoMultihopSettings();
  };

  test('Should display warning dialog when selecting "When needed" multihop mode with entry filters', async () => {
    const settings = getDefaultSettings();
    if ('normal' in settings.relaySettings) {
      const provider = relayList.countries[0].cities[0].relays[0].provider;
      settings.relaySettings.normal.wireguardConstraints.entryProviders = [provider];
      settings.relaySettings.normal.wireguardConstraints.multihop = 'always';
      settings.relaySettings.normal.wireguardConstraints.entryLocation = {
        only: {
          country: relayList.countries[0].code,
        },
      };
      await util.ipc.settings[''].notify(settings);
    }

    const multihopModeItem = routes.multihopSettings.getMultihopModeItem('When needed');
    await Promise.all([util.ipc.settings.setRelaySettings.ignore(), multihopModeItem.click()]);

    // Cancel first
    const cancelButton = routes.multihopSettings.getEntryFiltersOverrideDialogButton('Cancel');
    await expect(cancelButton).toBeVisible();
    await cancelButton.click();

    // Trigger dialog again
    await Promise.all([util.ipc.settings.setRelaySettings.ignore(), multihopModeItem.click()]);

    const continueButton = routes.multihopSettings.getEntryFiltersOverrideDialogButton('Continue');
    await expect(continueButton).toBeVisible();

    // Multihop mode should now be updated
    await Promise.all([util.ipc.settings.setRelaySettings.ignore(), continueButton.click()]);
  });
});
