import { expect, test } from '@playwright/test';
import { Page } from 'playwright';

import { MockedTestUtils, startMockedApp } from '../mocked-utils';
import { createHelpers, createIpc, createSelectors, mockData, resolveIpcHandle } from './helpers';

let page: Page;
let util: MockedTestUtils;
let helpers: ReturnType<typeof createHelpers>;
let ipc: ReturnType<typeof createIpc>;
let selectors: ReturnType<typeof createSelectors>;

test.describe('App upgrade', () => {
  if (process.platform === 'linux') {
    test.skip();
  }

  const startup = async () => {
    ({ page, util } = await startMockedApp());

    helpers = createHelpers(page, util);
    ipc = createIpc(util);
    selectors = createSelectors(page);

    await ipc.send.upgradeVersion({
      supported: true,
      suggestedIsBeta: false,
      suggestedUpgrade: {
        changelog: mockData.changelog,
        version: mockData.version,
      },
    });

    await util.waitForNavigation(() => page.click('button[aria-label="Settings"]'));
    await util.waitForNavigation(() => page.getByRole('button', { name: 'App info' }).click());
    await util.waitForNavigation(() =>
      page.getByRole('button', { name: 'Update available' }).click(),
    );
  };

  const restart = async () => {
    await page.close();
    await startup();
  };

  test.beforeAll(async () => {
    await startup();
  });

  test.afterAll(async () => {
    await page.close();
  });

  test.describe('Should display changelog', () => {
    test.afterAll(() => restart());

    test('Should display new version number as heading', async () => {
      const headingText = await page
        .getByRole('heading', {
          name: 'Version',
        })
        .textContent();
      expect(headingText).toBe(`Version ${mockData.version}`);
    });

    test('Should display new version changelog', async () => {
      const changelogList = page.getByRole('list');
      const changelogListText = await changelogList.textContent();
      expect(changelogListText).toEqual(mockData.changelog.join(''));

      const changelogListItems = page.getByRole('listitem');
      const changelogListItemsCount = await changelogListItems.count();
      expect(changelogListItemsCount).toBe(mockData.changelog.length);
    });
  });

  test.describe('Should download upgrade', () => {
    test.afterAll(() => restart());

    test('Should start upgrade when clicking Download & install button', async () => {
      await helpers.startAppUpgrade();
      const downloadAndInstallButton = selectors.downloadAndInstallButton();
      await expect(downloadAndInstallButton).toBeHidden();
    });

    test('Should show indeterminate download progress after upgrade started', async () => {
      await expect(page.getByText('Downloading...')).toBeVisible();
      await expect(page.getByText('Starting download...')).toBeVisible();

      await helpers.expectProgress(0, true);
    });

    test('Should show download progress after receiving event', async () => {
      const mockedProgress = 90;
      await ipc.send.appUpgradeEventDownloadProgress({
        progress: mockedProgress,
        server: 'cdn.mullvad.net',
        timeLeft: 120,
      });

      await expect(page.getByText('Downloading from: cdn.mullvad.net')).toBeVisible();
      await expect(page.getByText('About 2 minutes remaining...')).toBeVisible();

      await helpers.expectProgress(mockedProgress, true);
    });

    test('Should verify installer when download is complete', async () => {
      await ipc.send.appUpgradeEventVerifyingInstaller();

      await expect(page.getByText('Verifying installer')).toBeVisible();
      await expect(page.getByText('Download complete')).toBeVisible();

      await helpers.expectProgress(100);
    });

    test('Should show that it has verified the installer when verification is complete', async () => {
      await ipc.send.appUpgradeEventVerifiedInstaller();

      await expect(page.getByText('Verification successful! Starting installer...')).toBeVisible();
      await expect(page.getByText('Download complete')).toBeVisible();

      await helpers.expectProgress(100);
    });
  });

  test.describe('Should handle failing to download upgrade', () => {
    test.afterAll(() => restart());

    test('Should handle failing to download upgrade', async () => {
      await ipc.send.appUpgradeError('DOWNLOAD_FAILED');

      await expect(
        page.getByText(
          'Unable to download update. Check your connection and/or firewall then try again. If this problem persists, please contact support.',
        ),
      ).toBeVisible();

      const retryButton = selectors.retryButton();
      await expect(retryButton).toBeVisible();

      const reportProblemButton = selectors.reportProblemButton();
      await expect(reportProblemButton).toBeVisible();
    });

    test('Should handle retrying download of upgrade', async () => {
      const retryButton = selectors.retryButton();

      await resolveIpcHandle(ipc.handle.appUpgrade(), retryButton.click());

      await expect(page.getByText('Downloading...')).toBeVisible();
      await expect(page.getByText('Starting download...')).toBeVisible();

      await helpers.expectProgress(0, true);
    });
  });

  test.describe('Should handle failing to start installer', () => {
    test.afterAll(() => restart());

    test('Should handle failing to automatically start installer', async () => {
      await ipc.send.upgradeVersion({
        supported: true,
        suggestedIsBeta: false,
        suggestedUpgrade: {
          changelog: mockData.changelog,
          verifiedInstallerPath: mockData.verifiedInstallerPath,
          version: mockData.version,
        },
      });

      await ipc.send.appUpgradeEventVerifiedInstaller();
      await ipc.send.appUpgradeError('START_INSTALLER_AUTOMATIC_FAILED');

      await expect(page.getByText('Verification successful! Ready to install.')).toBeVisible();
    });

    test('Should handle failing to manually start installer', async () => {
      const installUpdateButton = selectors.installButton();

      await resolveIpcHandle(ipc.handle.appUpgrade(), installUpdateButton.click());

      await ipc.send.appUpgradeEventStartingInstaller();
      await ipc.send.appUpgradeError('START_INSTALLER_FAILED');

      await expect(installUpdateButton).not.toBeVisible();

      await expect(
        page.getByText(
          'Could not start the update installer, try downloading it again. If this problem persists, please contact support.',
        ),
      ).toBeVisible();
      const retryButton = selectors.retryButton();
      await expect(retryButton).toBeVisible();

      const reportProblemButton = selectors.reportProblemButton();
      await expect(reportProblemButton).toBeVisible();
    });

    test('Should handle retrying upgrade', async () => {
      const retryButton = selectors.retryButton();

      await resolveIpcHandle(ipc.handle.appUpgrade(), retryButton.click());

      await ipc.send.appUpgradeEventDownloadStarted();

      await expect(page.getByText('Downloading...')).toBeVisible();
      await expect(page.getByText('Starting download...')).toBeVisible();
      await helpers.expectProgress(0, true);
    });
  });

  test.describe('Should cancel download', () => {
    test('Should show Cancel button after upgrade started', async () => {
      await helpers.startAppUpgrade();
      const cancelButton = selectors.cancelButton();
      await expect(cancelButton).toBeVisible();
      await expect(cancelButton).toBeEnabled();
    });

    test('Should cancel upgrade when clicking the Cancel button', async () => {
      const cancelButton = selectors.cancelButton();

      await resolveIpcHandle(ipc.handle.appUpgradeAbort(), cancelButton.click());

      // After the app upgrade abort RPC is sent we expect to receive an aborted
      // event.
      await ipc.send.appUpgradeEventAborted();

      await expect(cancelButton).toBeHidden();

      const downloadAndInstallButton = selectors.downloadAndInstallButton();
      await expect(downloadAndInstallButton).toBeVisible();
    });

    test('Should start upgrade again when clicking Download & install button', async () => {
      await helpers.startAppUpgrade();

      const downloadAndInstallButton = selectors.downloadAndInstallButton();
      await expect(downloadAndInstallButton).toBeHidden();
    });
  });
});
