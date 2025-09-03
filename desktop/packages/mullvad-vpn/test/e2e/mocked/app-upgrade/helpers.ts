import { expect } from '@playwright/test';
import { Page } from 'playwright';

import { AppUpgradeEvent } from '../../../../src/shared/app-upgrade';
import { DaemonAppUpgradeEventStatusDownloadProgress } from '../../../../src/shared/daemon-rpc-types';
import { MockedTestUtils } from '../mocked-utils';

export const createAppUpgradeEventIpcHelper = (util: MockedTestUtils) => {
  const createMockResponseAppUpgradeEvent = (event: AppUpgradeEvent) =>
    util.ipc.app.upgradeEvent.notify(event);

  return {
    send: {
      appUpgradeEventAborted: () =>
        createMockResponseAppUpgradeEvent({
          type: 'APP_UPGRADE_STATUS_ABORTED',
        }),
      appUpgradeEventDownloadStarted: () =>
        createMockResponseAppUpgradeEvent({
          type: 'APP_UPGRADE_STATUS_DOWNLOAD_STARTED',
        }),
      appUpgradeEventDownloadProgress: (
        data: Omit<DaemonAppUpgradeEventStatusDownloadProgress, 'type'>,
      ) =>
        createMockResponseAppUpgradeEvent({
          type: 'APP_UPGRADE_STATUS_DOWNLOAD_PROGRESS',
          ...data,
        }),
      appUpgradeEventVerifyingInstaller: () =>
        createMockResponseAppUpgradeEvent({
          type: 'APP_UPGRADE_STATUS_VERIFYING_INSTALLER',
        }),
      appUpgradeEventVerifiedInstaller: () =>
        createMockResponseAppUpgradeEvent({
          type: 'APP_UPGRADE_STATUS_VERIFIED_INSTALLER',
        }),
      appUpgradeEventStartedInstaller: () =>
        createMockResponseAppUpgradeEvent({
          type: 'APP_UPGRADE_STATUS_STARTED_INSTALLER',
        }),
      appUpgradeEventExitedInstaller: () =>
        createMockResponseAppUpgradeEvent({
          type: 'APP_UPGRADE_STATUS_EXITED_INSTALLER',
        }),
    },
  };
};

export const createSelectors = (page: Page) => ({
  downloadAndLaunchInstallerButton: () =>
    page.getByRole('button', {
      name: 'Download & install',
    }),
  downloadProgressBar: () => page.getByRole('progressbar'),
  installButton: () =>
    page.getByRole('button', {
      name: 'Install update',
    }),
  manualDownloadLink: () =>
    page.getByRole('link', {
      name: 'Having problems? Try downloading the app from our website',
    }),
  pauseButton: () =>
    page.getByRole('button', {
      name: 'Pause',
    }),
  resumeButton: () =>
    page.getByRole('button', {
      name: 'Resume',
    }),
  retryButton: () =>
    page.getByRole('button', {
      name: 'Retry',
    }),
  reportProblemButton: () =>
    page.getByRole('button', {
      name: 'Report a problem',
    }),
  startingInstallerButton: () =>
    page.getByRole('button', {
      name: ' Starting installer...',
    }),
});

export const mockData = {
  changelog: ['This is a changelog.', 'Each item is on a separate line.', 'There are three items.'],
  verifiedInstallerPath: '/tmp/dummy-path',
  version: '2100.1',
};

export const createHelpers = (page: Page, util: MockedTestUtils) => {
  const selectors = createSelectors(page);
  const ipc = createAppUpgradeEventIpcHelper(util);

  const startAppUpgrade = async () => {
    const downloadAndLaunchInstallerButton = selectors.downloadAndLaunchInstallerButton();

    await Promise.all([util.ipc.app.upgrade.expect(), downloadAndLaunchInstallerButton.click()]);

    await ipc.send.appUpgradeEventDownloadStarted();
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

  return {
    expectProgress,
    startAppUpgrade,
  };
};
