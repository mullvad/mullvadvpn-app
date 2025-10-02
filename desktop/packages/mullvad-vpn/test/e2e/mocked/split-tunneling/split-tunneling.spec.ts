import { expect, test } from '@playwright/test';
import { Page } from 'playwright';

import { RoutePath } from '../../../../src/shared/routes';
import { RoutesObjectModel } from '../../route-object-models';
import { MockedTestUtils, startMockedApp } from '../mocked-utils';
import { linuxApplicationsList } from './helpers';

let page: Page;
let util: MockedTestUtils;
let routes: RoutesObjectModel;

const startup = async () => {
  ({ page, util } = await startMockedApp());
  routes = new RoutesObjectModel(page, util);

  await util.expectRoute(RoutePath.main);
  await routes.main.gotoSettings();
  await routes.settings.gotoSplitTunnelingSettings();
};

test.describe('Linux Split tunneling unsupported', () => {
  if (process.platform !== 'linux') {
    test.skip();
  }

  test.afterAll(async () => {
    await page.close();
  });

  test.beforeAll(async () => {
    await startup();
  });

  test.beforeAll(async () => {
    await util.ipc.linuxSplitTunneling.isSplitTunnelingSupported.handle(false);
    await util.ipc.linuxSplitTunneling.getApplications.handle(linuxApplicationsList);
  });

  test('App should show unsupported dialog when link in header is clicked', async () => {
    // Open the unsupported dialog
    await routes.splitTunnelingSettings.openUnsupportedDialog();
    const unsupportedText = routes.splitTunnelingSettings.getSplitTunnelingUnsupportedDialogText();
    await expect(unsupportedText).toBeVisible();

    // Close the unsupported dialog
    await routes.splitTunnelingSettings.closeUnsupportedDialog();
    await expect(unsupportedText).not.toBeVisible();
  });

  test('App list items should be shown even when split tunneling is unsupported', async () => {
    // Apps should be shown if split tunneling is unsupported
    const linuxApplications = routes.splitTunnelingSettings.getLinuxApplications();
    await expect(linuxApplications).toHaveCount(3);
  });

  test('App list items should show unsupported dialog when clicked', async () => {
    // Ensure clicking an application in the list makes the unsupported dialog visible
    const linuxApplications = routes.splitTunnelingSettings.getLinuxApplications();
    await linuxApplications.first().click();
    const unsupportedText = routes.splitTunnelingSettings.getSplitTunnelingUnsupportedDialogText();
    await expect(unsupportedText).toBeVisible();

    // Close the unsupported dialog
    await routes.splitTunnelingSettings.closeUnsupportedDialog();
    await expect(unsupportedText).not.toBeVisible();
  });
});

test.describe('Linux Split tunneling supported', () => {
  if (process.platform !== 'linux') {
    test.skip();
  }

  test.afterAll(async () => {
    await page.close();
  });

  test.beforeAll(async () => {
    await startup();
  });

  test.beforeAll(async () => {
    await util.ipc.linuxSplitTunneling.isSplitTunnelingSupported.handle(true);
    await util.ipc.linuxSplitTunneling.getApplications.handle(linuxApplicationsList);
  });

  test('App list items should be shown', async () => {
    const linuxApplications = routes.splitTunnelingSettings.getLinuxApplications();
    await expect(linuxApplications).toHaveCount(3);
  });

  test('App list items should be launched when clicked', async () => {
    // Launch the "app" application
    await Promise.all([
      util.ipc.linuxSplitTunneling.launchApplication.expect({ success: true }),
      routes.splitTunnelingSettings.openLinuxApplication('app'),
    ]);
  });

  test('App list items with warnings should show warning dialog when clicked', async () => {
    // Ensure clicking the application in the list makes the warning dialog visible
    await routes.splitTunnelingSettings.openLinuxApplication('launches-in-existing-process');

    const warningText = routes.splitTunnelingSettings.getLinuxApplicationWarningDialogText(
      'launches-in-existing-process',
    );
    await expect(warningText).toBeVisible();

    // Close the warning dialog
    await routes.splitTunnelingSettings.cancelLinuxApplicationWarningDialog();
    await expect(warningText).not.toBeVisible();
  });

  test('App list items should be filterered when searching', async () => {
    // List should be unfiltered at first
    const linuxApplications = routes.splitTunnelingSettings.getLinuxApplications();
    await expect(linuxApplications).toHaveCount(3);

    // List should only show 2 matching items
    await routes.splitTunnelingSettings.fillSearchInput('launches');
    await expect(linuxApplications).toHaveCount(2);
    let applicationNames = await linuxApplications.allInnerTexts();
    expect(applicationNames).toEqual(['launches-elsewhere', 'launches-in-existing-process']);

    // List should only show 1 matching item
    await routes.splitTunnelingSettings.fillSearchInput('app');
    await expect(linuxApplications).toHaveCount(1);
    applicationNames = await linuxApplications.allInnerTexts();
    expect(applicationNames).toEqual(['app']);

    // Clearing the search value should show all list items
    await routes.splitTunnelingSettings.clearSearchInput();
    await expect(linuxApplications).toHaveCount(3);
    applicationNames = await linuxApplications.allInnerTexts();
    expect(applicationNames).toEqual(['app', 'launches-elsewhere', 'launches-in-existing-process']);
  });

  test('App should launch file picker when button Find another app button is clicked', async () => {
    // Ensure clicking the "Find another app" button opens the file picker
    await Promise.all([
      util.ipc.app.showOpenDialog.expect({
        canceled: false,
        bookmarks: [],
        filePaths: [],
      }),
      routes.splitTunnelingSettings.openFindAnotherApp(),
    ]);

    // Ensure selecting an application with the file picker will launch the application
    await Promise.all([
      util.ipc.app.showOpenDialog.expect({
        canceled: false,
        bookmarks: [],
        filePaths: ['/app'],
      }),
      routes.splitTunnelingSettings.openFindAnotherApp(),
      util.ipc.linuxSplitTunneling.launchApplication.expect({
        success: true,
      }),
    ]);
  });
});
