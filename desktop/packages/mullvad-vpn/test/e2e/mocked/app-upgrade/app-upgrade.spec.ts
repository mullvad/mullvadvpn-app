import { expect, test } from '@playwright/test';
import { Page } from 'playwright';

import { RoutePath } from '../../../../src/shared/routes';
import { MockedTestUtils, startMockedApp } from '../mocked-utils';
import {
  createAppUpgradeEventIpcHelper,
  createHelpers,
  createSelectors,
  mockData,
} from './helpers';

let page: Page;
let util: MockedTestUtils;
let helpers: ReturnType<typeof createHelpers>;
let upgradeEventIpc: ReturnType<typeof createAppUpgradeEventIpcHelper>;
let selectors: ReturnType<typeof createSelectors>;

test.describe('App upgrade', () => {
  if (process.platform === 'linux') {
    test.skip();
  }

  const startup = async () => {
    ({ page, util } = await startMockedApp());

    helpers = createHelpers(page, util);
    upgradeEventIpc = createAppUpgradeEventIpcHelper(util);
    selectors = createSelectors(page);

    await util.expectRoute(RoutePath.main);

    await util.ipc.upgradeVersion[''].notify({
      supported: true,
      suggestedIsBeta: false,
      suggestedUpgrade: {
        changelog: mockData.changelog,
        version: mockData.version,
      },
    });

    await util.ipc.app.getUpgradeCacheDir.ignore();

    await page.click('button[aria-label="Settings"]');
    await util.expectRoute(RoutePath.settings);
    await page.getByRole('button', { name: 'App info' }).click();
    await util.expectRoute(RoutePath.appInfo);
    await page.getByRole('button', { name: 'Update available' }).click();
    await util.expectRoute(RoutePath.appUpgrade);
  };

  const restart = async () => {
    await util?.closePage();
    await startup();
  };

  test.beforeAll(async () => {
    await startup();
  });

  test.afterAll(async () => {
    await util?.closePage();
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
      const downloadAndLaunchInstallerButton = selectors.downloadAndLaunchInstallerButton();
      await expect(downloadAndLaunchInstallerButton).toBeHidden();
    });

    test('Should show indeterminate download progress after upgrade started', async () => {
      await expect(page.getByText('Downloading...')).toBeVisible();
      await expect(page.getByText('Starting download...')).toBeVisible();

      await helpers.expectProgress(0, true);
    });

    test('Should show download progress after receiving event', async () => {
      const mockedProgress = 90;
      await upgradeEventIpc.send.appUpgradeEventDownloadProgress({
        progress: mockedProgress,
        server: 'cdn.mullvad.net',
        timeLeft: 120,
      });

      await expect(page.getByText('Downloading from: cdn.mullvad.net')).toBeVisible();
      await expect(page.getByText('About 2 minutes remaining...')).toBeVisible();

      await helpers.expectProgress(mockedProgress, true);
    });

    test('Should verify installer when download is complete', async () => {
      await upgradeEventIpc.send.appUpgradeEventVerifyingInstaller();

      await expect(page.getByText('Verifying installer')).toBeVisible();
      await expect(page.getByText('Download complete')).toBeVisible();

      await helpers.expectProgress(100);
    });

    test('Should show that it has verified the installer when verification is complete', async () => {
      await upgradeEventIpc.send.appUpgradeEventVerifiedInstaller();

      await expect(page.getByText('Verification successful!')).toBeVisible();
      await expect(page.getByText('Download complete')).toBeVisible();

      await helpers.expectProgress(100);
    });
  });

  test.describe('Should handle failing to download upgrade', () => {
    test.afterAll(() => restart());

    test('Should handle failing to download upgrade', async () => {
      await util.ipc.app.upgradeError.notify('DOWNLOAD_FAILED');

      await expect(
        page.getByText(
          'Download failed, please check your connection/firewall and try again, or send a problem report.',
        ),
      ).toBeVisible();

      const retryButton = selectors.retryButton();
      await expect(retryButton).toBeVisible();

      const reportProblemButton = selectors.reportProblemButton();
      await expect(reportProblemButton).toBeVisible();
    });

    test('Should handle retrying download of upgrade', async () => {
      const retryButton = selectors.retryButton();

      await Promise.all([util.ipc.app.upgrade.expect(), retryButton.click()]);

      await expect(page.getByText('Downloading...')).toBeVisible();
      await expect(page.getByText('Starting download...')).toBeVisible();

      await helpers.expectProgress(0, true);
    });
  });

  test.describe('Should handle installer failing to start', () => {
    test.afterAll(() => restart());

    // This test should fail due to the window not being focused,
    // which is a pre-requisite for launching the installer automatically.
    test('Should handle installer failing to start automatically', async () => {
      await util.ipc.window.focus.notify(false);

      await util.ipc.upgradeVersion[''].notify({
        supported: true,
        suggestedIsBeta: false,
        suggestedUpgrade: {
          changelog: mockData.changelog,
          verifiedInstallerPath: mockData.verifiedInstallerPath,
          version: mockData.version,
        },
      });

      await upgradeEventIpc.send.appUpgradeEventVerifiedInstaller();

      const installUpdateButton = selectors.installButton();

      await expect(page.getByText('Verification successful!')).toBeVisible();
      await expect(installUpdateButton).toBeVisible();
      await expect(installUpdateButton).toBeEnabled();
    });

    test('Should handle installer failing to start manually', async () => {
      const installUpdateButton = selectors.installButton();

      await Promise.all([util.ipc.app.upgradeInstallerStart.expect(), installUpdateButton.click()]);

      await upgradeEventIpc.send.appUpgradeEventExitedInstaller();
      await util.ipc.app.upgradeError.notify('START_INSTALLER_FAILED');

      await expect(installUpdateButton).not.toBeVisible();

      await expect(
        page.getByText('Could not open installer, please try again or send a problem report.'),
      ).toBeVisible();
      const retryButton = selectors.retryButton();
      await expect(retryButton).toBeVisible();
      await expect(retryButton).toBeEnabled();

      const reportProblemButton = selectors.reportProblemButton();
      await expect(reportProblemButton).toBeVisible();
      await expect(reportProblemButton).toBeEnabled();
    });

    test('Should handle installer repeatedly failing to start', async () => {
      const retryButton = selectors.retryButton();

      // Call the retry button 2 additional times, to increase the total
      // errorCount to 3 in order for the ManualDownloadLink to be shown.
      await Promise.all([util.ipc.app.upgradeInstallerStart.expect(), retryButton.click()]);
      await upgradeEventIpc.send.appUpgradeEventExitedInstaller();
      await util.ipc.app.upgradeError.notify('START_INSTALLER_FAILED');

      await Promise.all([util.ipc.app.upgradeInstallerStart.expect(), retryButton.click()]);
      await upgradeEventIpc.send.appUpgradeEventExitedInstaller();
      await util.ipc.app.upgradeError.notify('START_INSTALLER_FAILED');

      const manualDownloadLink = selectors.manualDownloadLink();
      await expect(manualDownloadLink).toBeVisible();
    });
  });

  test.describe('Should pause download', () => {
    test('Should show Pause button after upgrade started', async () => {
      await helpers.startAppUpgrade();
      const pauseButton = selectors.pauseButton();

      await expect(pauseButton).toBeVisible();
      await expect(pauseButton).toBeEnabled();
    });

    test('Should pause upgrade when clicking the Pause button', async () => {
      const pauseButton = selectors.pauseButton();

      await Promise.all([util.ipc.app.upgradeAbort.expect(), pauseButton.click()]);

      // After the app upgrade abort RPC is sent we expect to receive an aborted
      // event.
      await upgradeEventIpc.send.appUpgradeEventAborted();

      await expect(pauseButton).toBeHidden();

      const resumeButton = selectors.resumeButton();
      await expect(resumeButton).toBeVisible();
      await expect(resumeButton).toBeEnabled();
    });

    test('Should start upgrade again when clicking Resume button', async () => {
      const resumeButton = selectors.resumeButton();

      await Promise.all([util.ipc.app.upgrade.expect(), resumeButton.click()]);

      await expect(resumeButton).toBeHidden();
    });
  });

  // NOTE: "Release halting" means that a release was made unavailable after it has been published,
  // likely due to a bug being detected after the release was published.
  //
  // When a release is halted it is no longer promoted in the app as a suggested upgrade.
  test.describe('Should handle release halting', () => {
    test.afterAll(() => restart());

    const releaseHalt = async () => {
      // Simulate release being halted
      await util.ipc.upgradeVersion[''].notify({
        supported: true,
        suggestedIsBeta: false,
        suggestedUpgrade: undefined,
      });
    };

    const releaseResume = async () => {
      // Simulate release being resumed
      await util.ipc.upgradeVersion[''].notify({
        supported: true,
        suggestedIsBeta: false,
        suggestedUpgrade: {
          changelog: mockData.changelog,
          version: mockData.version,
        },
      });
    };

    test('Should handle release being halted and resumed before upgrade started', async () => {
      await releaseHalt();

      await expect(page.getByText('You are using the latest version')).toBeVisible();
      await expect(page.getByText('Latest version: 2000.1')).toBeVisible();

      const downloadAndLaunchInstallerButton = selectors.downloadAndLaunchInstallerButton();
      await expect(downloadAndLaunchInstallerButton).toBeVisible();
      await expect(downloadAndLaunchInstallerButton).toBeDisabled();

      await releaseResume();
      await expect(downloadAndLaunchInstallerButton).toBeEnabled();
    });

    test('Should handle release being halted and resumed after upgrade started', async () => {
      await helpers.startAppUpgrade();

      await upgradeEventIpc.send.appUpgradeEventDownloadProgress({
        progress: 10,
        server: 'cdn.mullvad.net',
        timeLeft: 120,
      });

      await releaseHalt();

      await expect(page.getByText('You are using the latest version')).toBeVisible();
      await expect(page.getByText('Latest version: 2000.1')).toBeVisible();

      const downloadAndLaunchInstallerButton = selectors.downloadAndLaunchInstallerButton();
      await expect(downloadAndLaunchInstallerButton).toBeVisible();
      await expect(downloadAndLaunchInstallerButton).toBeDisabled();

      await releaseResume();

      // User should now be able to restart the upgrade
      await helpers.startAppUpgrade();
    });
  });
});
