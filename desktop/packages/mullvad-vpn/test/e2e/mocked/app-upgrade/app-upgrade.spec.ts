import { expect, test } from '@playwright/test';
import { Page } from 'playwright';

import { AppUpgradeError, AppUpgradeEvent } from '../../../../src/shared/app-upgrade';
import { IAppVersionInfo } from '../../../../src/shared/daemon-rpc-types';
import { MockedTestUtils, startMockedApp } from '../mocked-utils';

let page: Page;
let util: MockedTestUtils;

test.describe('App upgrade', () => {
  const mockedVersion = '2100.1';
  const mockedChangelog = [
    'This is a changelog.',
    'Each item is on a separate line.',
    'There are three items.',
  ];

  test.beforeAll(async () => {
    ({ page, util } = await startMockedApp());

    await util.sendMockIpcResponse<IAppVersionInfo>({
      channel: 'upgradeVersion-',
      response: {
        supported: true,
        suggestedIsBeta: false,
        suggestedUpgrade: {
          version: mockedVersion,
          changelog: mockedChangelog,
        },
      },
    });

    await util.waitForNavigation(() => page.click('button[aria-label="Settings"]'));
    await util.waitForNavigation(() => page.getByRole('button', { name: 'App info' }).click());
    await util.waitForNavigation(() =>
      page.getByRole('button', { name: 'Update available' }).click(),
    );
  });

  test.afterAll(async () => {
    await page.close();
  });

  const resolveIpcHandle = async (test: Promise<void>, trigger: Promise<void>) => {
    // The promise is resolved when its handle has been called.
    // The handle should be called when the trigger is called.
    const promise = await Promise.all([test, trigger]);
    expect(promise).toBeTruthy();
  };

  const selectors = {
    downloadAndInstallButton: () =>
      page.getByRole('button', {
        name: 'Download and install',
      }),
    installButton: () =>
      page.getByRole('button', {
        name: 'Install update',
      }),
    cancelButton: () =>
      page.getByRole('button', {
        name: 'Cancel',
      }),
    retryButton: () =>
      page.getByRole('button', {
        name: 'Retry download',
      }),
    reportProblemButton: () =>
      page.getByRole('button', {
        name: 'Report a problem',
      }),
    downloadProgressBar: () => page.getByRole('progressbar'),
  } as const;

  const startAppUpgrade = async () => {
    const downloadAndInstallButton = selectors.downloadAndInstallButton();

    await resolveIpcHandle(
      util.mockIpcHandle({
        channel: 'appUpgrade',
        response: undefined,
      }),
      downloadAndInstallButton.click(),
    );

    // Mock that we have started downloading the upgrade
    await util.sendMockIpcResponse<AppUpgradeEvent>({
      channel: 'app-upgradeEvent',
      response: {
        type: 'APP_UPGRADE_STATUS_DOWNLOAD_STARTED',
      },
    });
  };

  const expectProgress = async (progress: number, expectLabel?: boolean) => {
    if (expectLabel) {
      await expect(page.getByText(`${progress}%`)).toBeVisible();
    }

    const downloadProgressBarValue = await selectors
      .downloadProgressBar()
      .getAttribute('aria-valuenow');
    expect(downloadProgressBarValue).toEqual(progress.toString());
  };

  test.describe('Should display changelog', () => {
    test('Should display new version number as heading', async () => {
      const headingText = await page
        .getByRole('heading', {
          name: 'Version',
        })
        .textContent();
      expect(headingText).toBe(`Version ${mockedVersion}`);
    });

    test('Should display new version changelog', async () => {
      const changelogList = page.getByRole('list');
      const changelogListText = await changelogList.textContent();
      expect(changelogListText).toEqual(mockedChangelog.join(''));

      const changelogListItems = page.getByRole('listitem');
      const changelogListItemsCount = await changelogListItems.count();
      expect(changelogListItemsCount).toBe(mockedChangelog.length);
    });
  });

  test.describe('Should download upgrade', () => {
    test('Should start upgrade when clicking Download & install button', async () => {
      await startAppUpgrade();
      const downloadAndInstallButton = selectors.downloadAndInstallButton();
      await expect(downloadAndInstallButton).toBeHidden();
    });

    test('Should show indeterminate download progress after upgrade started', async () => {
      // TODO: Improve by using aria labels
      await expect(page.getByText('Downloading...')).toBeVisible();
      await expect(page.getByText('Starting download...')).toBeVisible();

      await expectProgress(0, true);
    });

    test('Should show download progress after receiving event', async () => {
      const mockedProgress = 90;
      // Mock that we have started downloading the upgrade
      await util.sendMockIpcResponse<AppUpgradeEvent>({
        channel: 'app-upgradeEvent',
        response: {
          type: 'APP_UPGRADE_STATUS_DOWNLOAD_PROGRESS',
          progress: mockedProgress,
          server: 'cdn.mullvad.net',
          timeLeft: 120,
        },
      });

      await expect(page.getByText('Downloading from: cdn.mullvad.net')).toBeVisible();
      await expect(page.getByText('About 2 minutes remaining...')).toBeVisible();

      await expectProgress(mockedProgress, true);
    });

    test('Should verify installer when download is complete', async () => {
      // Mock that we have started verifying the upgrade
      await util.sendMockIpcResponse<AppUpgradeEvent>({
        channel: 'app-upgradeEvent',
        response: {
          type: 'APP_UPGRADE_STATUS_VERIFYING_INSTALLER',
        },
      });

      await expect(page.getByText('Verifying installer')).toBeVisible();
      await expect(page.getByText('Download complete')).toBeVisible();

      await expectProgress(100);
    });

    test('Should show that it has verified the installer when verification is complete', async () => {
      // Mock that we have verified the upgrade
      await util.sendMockIpcResponse<AppUpgradeEvent>({
        channel: 'app-upgradeEvent',
        response: {
          type: 'APP_UPGRADE_STATUS_VERIFIED_INSTALLER',
        },
      });

      await expect(page.getByText('Verification successful! Starting installer...')).toBeVisible();
      await expect(page.getByText('Download complete')).toBeVisible();

      await expectProgress(100);
    });
  });

  test.describe('Should handle failing to download upgrade', () => {
    test('Should handle failing to download upgrade', async () => {
      // Mock that we have encountered an error the upgrade
      await util.sendMockIpcResponse<AppUpgradeError>({
        channel: 'app-upgradeError',
        response: 'DOWNLOAD_FAILED',
      });

      await expect(
        page.getByText(
          'Unable to download update. Check your connection and/or firewall then try again. If this problem persists, please contact support.',
        ),
      ).toBeVisible();

      const retryDownloadButton = selectors.retryButton();
      await expect(retryDownloadButton).toBeVisible();

      const reportProblemButton = selectors.reportProblemButton();
      await expect(reportProblemButton).toBeVisible();
    });
    test('Should handle retrying downloading upgrade', async () => {
      const retryDownloadButton = selectors.retryButton();

      await resolveIpcHandle(
        util.mockIpcHandle({
          channel: 'appUpgrade',
          response: undefined,
        }),
        retryDownloadButton.click(),
      );

      await expect(page.getByText('Downloading...')).toBeVisible();
      await expect(page.getByText('Starting download...')).toBeVisible();
      await expectProgress(0, true);
    });
  });

  test.describe('Should handle failing to start installer', () => {
    test('Should handle failing to automatically start installer', async () => {
      // Mock the verified upgrade installer path
      await util.sendMockIpcResponse<IAppVersionInfo>({
        channel: 'upgradeVersion-',
        response: {
          supported: true,
          suggestedIsBeta: false,
          suggestedUpgrade: {
            version: mockedVersion,
            changelog: mockedChangelog,
            verifiedInstallerPath: '/tmp/dummy-path',
          },
        },
      });

      await util.sendMockIpcResponse<AppUpgradeError>({
        channel: 'app-upgradeError',
        response: 'START_INSTALLER_AUTOMATIC_FAILED',
      });

      await expect(page.getByText('Verification successful! Ready to install.')).toBeVisible();

      const downloadProgressBar = selectors.downloadProgressBar();
      await expect(downloadProgressBar).not.toBeVisible();
    });

    test('Should handle failing to manually start installer', async () => {
      const installUpdateButton = selectors.installButton();

      await resolveIpcHandle(
        util.mockIpcHandle({
          channel: 'appUpgrade',
          response: undefined,
        }),
        installUpdateButton.click(),
      );

      // Mock that we have encountered an error the upgrade
      await util.sendMockIpcResponse<AppUpgradeError>({
        channel: 'app-upgradeError',
        response: 'START_INSTALLER_FAILED',
      });

      await expect(installUpdateButton).not.toBeVisible();

      await expect(
        page.getByText(
          'Could not start the update installer, try downloading it again. If this problem persists, please contact support.',
        ),
      ).toBeVisible();
      const retryDownloadButton = selectors.retryButton();
      await expect(retryDownloadButton).toBeVisible();

      const reportProblemButton = selectors.reportProblemButton();
      await expect(reportProblemButton).toBeVisible();
    });

    test('Should handle retrying downloading upgrade', async () => {
      const retryDownloadButton = selectors.retryButton();

      await resolveIpcHandle(
        util.mockIpcHandle({
          channel: 'appUpgrade',
          response: undefined,
        }),
        retryDownloadButton.click(),
      );

      await expect(page.getByText('Downloading...')).toBeVisible();
      await expect(page.getByText('Starting download...')).toBeVisible();
      await expectProgress(0, true);
    });
  });

  test.describe('Should cancel download', () => {
    test('Should show Cancel button after upgrade started', async () => {
      await startAppUpgrade();
      const cancelButton = selectors.cancelButton();
      await expect(cancelButton).toBeVisible();
      await expect(cancelButton).toBeEnabled();
    });

    test('Should cancel upgrade when clicking the Cancel button', async () => {
      const cancelButton = selectors.cancelButton();

      await resolveIpcHandle(
        util.mockIpcHandle({
          channel: 'appUpgradeAbort',
          response: undefined,
        }),
        cancelButton.click(),
      );

      // After the app upgrade abort RPC is sent we expect to receive an aborted
      // event.
      await util.sendMockIpcResponse<AppUpgradeEvent>({
        channel: 'app-upgradeEvent',
        response: {
          type: 'APP_UPGRADE_STATUS_ABORTED',
        },
      });

      // The cancel button should be hidden when the upgrade is aborted
      await expect(cancelButton).toBeHidden();

      // The Download & install button should become visible again
      const downloadAndInstallButton = selectors.downloadAndInstallButton();
      await expect(downloadAndInstallButton).toBeVisible();
    });

    test('Should start upgrade again when clicking Download & install button', async () => {
      await startAppUpgrade();
      const downloadAndInstallButton = selectors.downloadAndInstallButton();

      await expect(downloadAndInstallButton).toBeHidden();
    });
  });
});
