import { expect, test } from '@playwright/test';
import { Page } from 'playwright';

import { IDevice } from '../../../../src/shared/daemon-rpc-types';
import { RoutesObjectModel } from '../../route-object-models';
import { MockedTestUtils, startMockedApp } from '../mocked-utils';
import { createHelpers, MAnageDevicesHelpers as ManageDevicesHelpers } from './helpers';

let page: Page;
let util: MockedTestUtils;
let routes: RoutesObjectModel;
let helpers: ManageDevicesHelpers;

export const mockDevices: IDevice[] = [
  { id: '1', name: 'Sneaky dog', created: new Date('2024-12-05') },
  { id: '2', name: 'Wise cat', created: new Date('2025-01-14') },
  { id: '3', name: 'Cool panda', created: new Date('2025-03-22') },
  { id: '4', name: 'Strong fish', created: new Date('2025-06-01') },
  { id: '5', name: 'Magic elk', created: new Date('2025-09-10') },
];

test.describe('Manage devices view', () => {
  const currentDevice = mockDevices[0];

  test.beforeAll(async () => {
    ({ page, util } = await startMockedApp());
    routes = new RoutesObjectModel(page, util);
    helpers = createHelpers(util);

    await routes.main.waitForRoute();
    await helpers.setCurrentDevice(currentDevice);
    await routes.main.gotoAccount();
    await routes.account.gotoManageDevices();
  });

  test.beforeEach(async () => {
    await helpers.setDevices(mockDevices);
  });

  test.afterAll(async () => {
    await page.close();
  });

  test('Should display all account devices', async () => {
    const deviceListItems = routes.manageDevices.selectors.deviceListItems();
    for (const device of mockDevices) {
      await expect(deviceListItems.filter({ hasText: device.name })).toBeVisible();
    }
  });

  test('Should be able to remove all but current device', async () => {
    for (const device of mockDevices) {
      if (device.id === currentDevice.id) {
        continue;
      }
      const deviceItem = routes.manageDevices.selectors.deviceListItem(device.name);
      await expect(deviceItem).toBeVisible();

      const removeButton = routes.manageDevices.selectors.removeDeviceButton(device.name);
      await expect(removeButton).toBeVisible();
    }
  });

  test('Should be able to delete other device', async () => {
    const deviceToRemove = mockDevices[1];

    const deviceItem = routes.manageDevices.selectors.deviceListItem(deviceToRemove.name);
    const removeButton = routes.manageDevices.selectors.removeDeviceButton(deviceToRemove.name);
    await removeButton.click();

    const confirmButton = routes.manageDevices.selectors.confirmRemoveDeviceButton();
    await Promise.all([util.ipc.account.removeDevice.expect(), confirmButton.click()]);
    await helpers.setDevices(mockDevices.filter((d) => d.name !== deviceToRemove.name)),
      await expect(deviceItem).toHaveCount(0);
  });

  test('Should not be able to delete current device', async () => {
    const deviceListItems = routes.manageDevices.selectors.removeDeviceButton(currentDevice.name);
    await expect(deviceListItems).toHaveCount(0);
  });
});
