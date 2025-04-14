import { expect } from '@playwright/test';
import { Page } from 'playwright';

import { AppUpgradeError, AppUpgradeEvent } from '../../../../src/shared/app-upgrade';
import {
  DaemonAppUpgradeEventStatusDownloadProgress,
  IAppVersionInfo,
} from '../../../../src/shared/daemon-rpc-types';
import { MockedTestUtils } from '../mocked-utils';

export const createIpc = (util: MockedTestUtils) => {
  const createMockHandle = <T>(channel: string, response?: T) =>
    util.mockIpcHandle<T | undefined>({ channel, response });

  const createMockResponse = <T>(channel: string, response: T) =>
    util.sendMockIpcResponse<T>({
      channel,
      response,
    });

  const createMockResponseAppUpgradeEvent = (event: AppUpgradeEvent) =>
    createMockResponse<AppUpgradeEvent>('app-upgradeEvent', event);

  return {
    handle: {
      appUpgrade: () => createMockHandle('appUpgrade'),
      appUpgradeAbort: () => createMockHandle('appUpgradeAbort'),
    },
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
      appUpgradeEventStartingInstaller: () =>
        createMockResponseAppUpgradeEvent({
          type: 'APP_UPGRADE_STATUS_STARTING_INSTALLER',
        }),
      appUpgradeEventStartedInstaller: () =>
        createMockResponseAppUpgradeEvent({
          type: 'APP_UPGRADE_STATUS_STARTED_INSTALLER',
        }),
      appUpgradeEventExitedInstaller: () =>
        createMockResponseAppUpgradeEvent({
          type: 'APP_UPGRADE_STATUS_EXITED_INSTALLER',
        }),
      appUpgradeError: (error: AppUpgradeError) =>
        createMockResponse<AppUpgradeError>('app-upgradeError', error),
      upgradeVersion: (data: IAppVersionInfo) =>
        createMockResponse<IAppVersionInfo>('upgradeVersion-', data),
    },
  };
};

export const createSelectors = (page: Page) => ({
  downloadAndInstallButton: () =>
    page.getByRole('button', {
      name: 'Download & install',
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
      name: 'Retry',
    }),
  reportProblemButton: () =>
    page.getByRole('button', {
      name: 'Report a problem',
    }),
  downloadProgressBar: () => page.getByRole('progressbar'),
});

export const mockData = {
  changelog: ['This is a changelog.', 'Each item is on a separate line.', 'There are three items.'],
  verifiedInstallerPath: '/tmp/dummy-path',
  version: '2100.1',
};

export const resolveIpcHandle = async (test: Promise<void>, trigger: Promise<void>) => {
  // The promise is resolved when its handle has been called.
  // The handle should be called when the trigger is called.
  const promise = await Promise.all([test, trigger]);
  expect(promise).toBeTruthy();
};

export const createHelpers = (page: Page, util: MockedTestUtils) => {
  const selectors = createSelectors(page);
  const ipc = createIpc(util);

  const testTeardown = async () => {
    await ipc.send.appUpgradeEventAborted();
    await ipc.send.upgradeVersion({
      supported: true,
      suggestedIsBeta: false,
      suggestedUpgrade: {
        changelog: mockData.changelog,
        version: mockData.version,
      },
    });
  };

  const startAppUpgrade = async () => {
    const downloadAndInstallButton = selectors.downloadAndInstallButton();

    await resolveIpcHandle(ipc.handle.appUpgrade(), downloadAndInstallButton.click());

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
    testTeardown,
  };
};
