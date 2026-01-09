import { expect, test } from '@playwright/test';
import { Page } from 'playwright';

import { IGuiSettingsState } from '../../../../src/shared/gui-settings-state';
import { RoutesObjectModel } from '../../route-object-models';
import { MockedTestUtils, startMockedApp } from '../mocked-utils';

let page: Page;
let util: MockedTestUtils;
let routes: RoutesObjectModel;

test.describe('User interface settings', () => {
  const startup = async () => {
    ({ page, util } = await startMockedApp());
    routes = new RoutesObjectModel(page, util);

    await routes.main.waitForRoute();

    await routes.main.gotoSettings();
    await routes.settings.gotoUserInterfaceSettings();
  };

  const setGuiSettings = async (state: Partial<IGuiSettingsState> = {}) => {
    const baseState: IGuiSettingsState = {
      enableSystemNotifications: false,
      monochromaticIcon: false,
      startMinimized: false,
      animateMap: false,
      autoConnect: false,
      unpinnedWindow: false,
      browsedForSplitTunnelingApplications: [],
      changelogDisplayedForVersion: '',
      preferredLocale: 'en',
      updateDismissedForVersion: '',
    };

    await util.ipc.guiSettings[''].notify({
      ...baseState,
      ...state,
    });
  };

  test.beforeAll(async () => {
    await startup();
  });

  test.afterAll(async () => {
    await util?.closePage();
  });

  test.beforeEach(async () => {
    await setGuiSettings();
  });

  test.describe('Notification settings', () => {
    test('Should toggle notification setting', async () => {
      const autoStartSwitch = routes.userInterfaceSettings.selectors.autoStartSwitch();
      await expect(autoStartSwitch).toBeVisible();
      await expect(autoStartSwitch).not.toBeChecked();

      await Promise.all([
        autoStartSwitch.click(),
        util.ipc.guiSettings.setEnableSystemNotifications.expect(),
      ]);

      await setGuiSettings({ enableSystemNotifications: true });
      await expect(autoStartSwitch).toBeChecked();
    });
  });

  test.describe('Monochromatic tray icon settings', () => {
    test('Should toggle monochromatic tray icon setting', async () => {
      const monochromaticTrayIconSwitch =
        routes.userInterfaceSettings.selectors.monochromaticTrayIconSwitch();
      await expect(monochromaticTrayIconSwitch).toBeVisible();
      await expect(monochromaticTrayIconSwitch).not.toBeChecked();

      await Promise.all([
        monochromaticTrayIconSwitch.click(),
        util.ipc.guiSettings.setMonochromaticIcon.expect(),
      ]);

      await setGuiSettings({ monochromaticIcon: true });
      await expect(monochromaticTrayIconSwitch).toBeChecked();
    });
  });

  test.describe('Unpinned window setting', () => {
    test.skip(() => process.platform !== 'win32');

    test('Should toggle unpinned window setting', async () => {
      const unpinnedWindowSwitch = routes.userInterfaceSettings.selectors.unpinnedWindowSwitch();
      await expect(unpinnedWindowSwitch).toBeVisible();
      await expect(unpinnedWindowSwitch).not.toBeChecked();

      await Promise.all([
        unpinnedWindowSwitch.click(),
        util.ipc.guiSettings.setUnpinnedWindow.expect(),
      ]);

      await setGuiSettings({ unpinnedWindow: true });
      await expect(unpinnedWindowSwitch).toBeChecked();
    });
  });

  test.describe('Start minimized setting', () => {
    test.skip(() => process.platform !== 'win32');

    test('Should toggle start minimized setting', async () => {
      await setGuiSettings({ unpinnedWindow: true });

      const startMinimizedSwitch = routes.userInterfaceSettings.selectors.startMinimizedSwitch();
      await expect(startMinimizedSwitch).toBeVisible();
      await expect(startMinimizedSwitch).not.toBeChecked();

      await Promise.all([
        startMinimizedSwitch.click(),
        util.ipc.guiSettings.setStartMinimized.expect(),
      ]);

      await setGuiSettings({ unpinnedWindow: true, startMinimized: true });
      await expect(startMinimizedSwitch).toBeChecked();
    });
  });

  test.describe('Animate map setting', () => {
    test.describe('With reduced motion', () => {
      test.beforeEach(async () => {
        await util.setReducedMotion('reduce');
      });

      test('Should not display animate map setting', async () => {
        const animateMapSwitch = routes.userInterfaceSettings.selectors.animateMapSwitch();
        await expect(animateMapSwitch).not.toBeVisible();
      });
    });

    test.describe('Without reduced motion', () => {
      test.beforeEach(async () => {
        await util.setReducedMotion('no-preference');
      });

      test('Should display animate map setting', async () => {
        const animateMapSwitch = routes.userInterfaceSettings.selectors.animateMapSwitch();

        await expect(animateMapSwitch).toBeVisible();
      });

      test('Should toggle animate map setting', async () => {
        const animateMapSwitch = routes.userInterfaceSettings.selectors.animateMapSwitch();
        await expect(animateMapSwitch).toBeVisible();
        await expect(animateMapSwitch).not.toBeChecked();

        await Promise.all([animateMapSwitch.click(), util.ipc.guiSettings.setAnimateMap.expect()]);

        await setGuiSettings({ animateMap: true });
        await expect(animateMapSwitch).toBeChecked();
      });
    });
  });

  test.describe('Select language', () => {
    ['Svenska', 'Deutsch', 'English', 'System default'].forEach((language) => {
      test(`Should change language to ${language}`, async () => {
        await routes.userInterfaceSettings.gotoSelectLanguage();
        await routes.selectLanguage.selectLanguage(language);
        await routes.selectLanguage.goBack();

        await expect(
          routes.userInterfaceSettings.getLocalizedLanguageButton(language),
        ).toBeVisible();
      });
    });
  });
});
