import { expect, test } from '@playwright/test';
import { Page } from 'playwright';

import { AppUpgradeError, AppUpgradeEvent } from '../../../src/shared/app-upgrade';
import { IAppVersionInfo } from '../../../src/shared/daemon-rpc-types';
import { MockedTestUtils, startMockedApp } from './mocked-utils';

let page: Page;
let util: MockedTestUtils;

test.beforeAll(async () => {
  ({ page, util } = await startMockedApp());

  await util.sendMockIpcResponse<IAppVersionInfo>({
    channel: 'upgradeVersion-',
    response: {
      supported: true,
      suggestedIsBeta: false,
      suggestedUpgrade: {
        version: '2100.1',
        changelog: [
          'This is a changelog.',
          'Each item is on a separate line.',
          'There are three items.',
        ],
      },
    },
  });
});

test.afterEach(async () => {
  await new Promise((resolve) => {
    setTimeout(() => {
      resolve(null);
    }, 5000);
  });
});

test.afterAll(async () => {
  await page.close();
});

test('App should navigate to App upgrade view', async () => {
  await util.waitForNavigation(() => page.click('button[aria-label="Settings"]'));
  await util.waitForNavigation(() => page.getByRole('button', { name: 'App info' }).click());
  await util.waitForNavigation(() =>
    page.getByRole('button', { name: 'Update available' }).click(),
  );
});

test('App should display version and changelog of new upgrade', async () => {
  const headingText = await page
    .getByRole('heading', {
      name: 'Version',
    })
    .textContent();
  expect(headingText).toBe('Version 2100.1');

  const changelogList = page.getByRole('list');
  const changelogListText = await changelogList.textContent();
  expect(changelogListText).toEqual(
    'This is a changelog.Each item is on a separate line.There are three items.',
  );

  const changelogListItems = page.getByRole('listitem');
  const changelogListItemsCount = await changelogListItems.count();
  expect(changelogListItemsCount).toBe(3);
});

test('App should start upgrade when clicking Download & install button', async () => {
  const downloadAndInstallButton = page.getByRole('button', {
    name: 'Download and install',
  });

  const appUpgradePromise = util.mockIpcHandle({
    channel: 'appUpgrade',
    response: undefined,
  });

  await downloadAndInstallButton.click();

  // The appUpgrade promise is resolved when its handle has been called.
  // The handle should be called when the Download & install button is clicked.
  await appUpgradePromise;

  // Mock that we have started downloading the upgrade
  await util.sendMockIpcResponse<AppUpgradeEvent>({
    channel: 'app-upgradeEvent',
    response: {
      type: 'APP_UPGRADE_STATUS_DOWNLOAD_STARTED',
    },
  });

  await expect(downloadAndInstallButton).toBeHidden();
});

test('App should show Cancel button after upgrade started', async () => {
  const cancelButton = page.getByRole('button', {
    name: 'Cancel',
  });
  await expect(cancelButton).toBeVisible();
  await expect(cancelButton).toBeEnabled();
});

test('App should show indeterminate download progress after upgrade started', async () => {
  // TODO: Improve by using aria labels
  await expect(page.getByText('Downloading...')).toBeVisible();
  await expect(page.getByText('Starting download...')).toBeVisible();
  await expect(page.getByText('0%')).toBeVisible();

  const downloadProgressBarValue = await page
    .getByRole('progressbar')
    .getAttribute('aria-valuenow');
  expect(downloadProgressBarValue).toEqual('0');
});

test('App should show download progress after receiving event', async () => {
  // Mock that we have started downloading the upgrade
  await util.sendMockIpcResponse<AppUpgradeEvent>({
    channel: 'app-upgradeEvent',
    response: {
      type: 'APP_UPGRADE_STATUS_DOWNLOAD_PROGRESS',
      progress: 90,
      server: 'cdn.mullvad.net',
      timeLeft: 120,
    },
  });

  // TODO: Improve by using aria labels
  await expect(page.getByText('Downloading from: cdn.mullvad.net')).toBeVisible();
  await expect(page.getByText('About 2 minutes remaining...')).toBeVisible();

  const downloadProgressBarValue = await page
    .getByRole('progressbar')
    .getAttribute('aria-valuenow');
  expect(downloadProgressBarValue).toEqual('90');
});

test('App should cancel upgrade when clicking the Cancel button', async () => {
  const cancelButton = page.getByRole('button', {
    name: 'Cancel',
  });

  const appUpgradeAbortPromise = util.mockIpcHandle({
    channel: 'appUpgradeAbort',
    response: undefined,
  });

  await cancelButton.click();

  // The appUpgradeAbort promise is resolved when its handle has been called.
  // The handle should be called when the cancel button is clicked.
  await appUpgradeAbortPromise;

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
  const downloadAndInstallButton = page.getByRole('button', {
    name: 'Download and install',
  });
  await expect(downloadAndInstallButton).toBeVisible();
});

test('App should start upgrade again when clicking Download & install button', async () => {
  const downloadAndInstallButton = page.getByRole('button', {
    name: 'Download and install',
  });

  const appUpgradePromise = util.mockIpcHandle({
    channel: 'appUpgrade',
    response: undefined,
  });

  await downloadAndInstallButton.click();

  // The appUpgrade promise is resolved when its handle has been called.
  // The handle should be called when the Download & install button is clicked.
  await appUpgradePromise;

  // Mock that we have started downloading the upgrade
  await util.sendMockIpcResponse<AppUpgradeEvent>({
    channel: 'app-upgradeEvent',
    response: {
      type: 'APP_UPGRADE_STATUS_DOWNLOAD_STARTED',
    },
  });

  await expect(downloadAndInstallButton).toBeHidden();
});

test('App should show that it is verifying the installer when download is complete', async () => {
  // Mock that we have started verifying the upgrade
  await util.sendMockIpcResponse<AppUpgradeEvent>({
    channel: 'app-upgradeEvent',
    response: {
      type: 'APP_UPGRADE_STATUS_VERIFYING_INSTALLER',
    },
  });

  // TODO: Improve by using aria labels
  await expect(page.getByText('Verifying installer')).toBeVisible();
  await expect(page.getByText('Download complete')).toBeVisible();

  const downloadProgressBarValue = await page
    .getByRole('progressbar')
    .getAttribute('aria-valuenow');
  expect(downloadProgressBarValue).toEqual('100');
});

test('App should show that it has verified the installer when verification is complete', async () => {
  // Mock that we have verified the upgrade
  await util.sendMockIpcResponse<AppUpgradeEvent>({
    channel: 'app-upgradeEvent',
    response: {
      type: 'APP_UPGRADE_STATUS_VERIFIED_INSTALLER',
    },
  });

  // TODO: Improve by using aria labels
  await expect(page.getByText('Verification successful! Starting installer...')).toBeVisible();
  await expect(page.getByText('Download complete')).toBeVisible();

  const downloadProgressBarValue = await page
    .getByRole('progressbar')
    .getAttribute('aria-valuenow');
  expect(downloadProgressBarValue).toEqual('100');
});

test('App should handle failing to automatically start installer', async () => {
  // Mock the verified upgrade installer path
  await util.sendMockIpcResponse<IAppVersionInfo>({
    channel: 'upgradeVersion-',
    response: {
      supported: true,
      suggestedIsBeta: false,
      suggestedUpgrade: {
        version: '2100.1',
        changelog: [
          'This is a changelog.',
          'Each item is on a separate line.',
          'There are three items.',
        ],
        verifiedInstallerPath: '/tmp/dummy-path',
      },
    },
  });

  await util.sendMockIpcResponse<AppUpgradeError>({
    channel: 'app-upgradeError',
    response: 'START_INSTALLER_AUTOMATIC_FAILED',
  });

  // TODO: Improve by using aria labels
  await expect(page.getByText('Verification successful! Ready to install.')).toBeVisible();

  const downloadProgressBar = page.getByRole('progressbar');
  await expect(downloadProgressBar).not.toBeVisible();
});

test('App should handle failing to manually start installer', async () => {
  const installUpdateButton = page.getByRole('button', {
    name: 'Install update',
  });

  const appUpgradePromise = util.mockIpcHandle({
    channel: 'appUpgrade',
    response: undefined,
  });

  await installUpdateButton.click();

  // The appUpgrade promise is resolved when its handle has been called.
  // The handle should be called when the Install update button is clicked.
  await appUpgradePromise;

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
  const retryDownloadButton = page.getByRole('button', {
    name: 'Retry download',
  });
  await expect(retryDownloadButton).toBeVisible();

  const reportProblemButton = page.getByRole('button', {
    name: 'Report a problem',
  });
  await expect(reportProblemButton).toBeVisible();
});

test('App should handle retrying downloading the update', async () => {
  const retryDownloadButton = page.getByRole('button', {
    name: 'Retry download',
  });

  const appUpgradePromise = util.mockIpcHandle({
    channel: 'appUpgrade',
    response: undefined,
  });

  await retryDownloadButton.click();

  // The appUpgrade promise is resolved when its handle has been called.
  // The handle should be called when the Retry download button is clicked.
  await appUpgradePromise;

  // TODO: Improve by using aria labels
  await expect(page.getByText('Downloading...')).toBeVisible();
  await expect(page.getByText('Starting download...')).toBeVisible();
  await expect(page.getByText('0%')).toBeVisible();

  const downloadProgressBarValue = await page
    .getByRole('progressbar')
    .getAttribute('aria-valuenow');
  expect(downloadProgressBarValue).toEqual('0');
});

test('App should handle starting installer again if installer was started but did not close the app', async () => {
  // Mock that we have started the installer. In the real app this event is sent
  // after a timeout if the GUI is still running after the installer has been started
  await util.sendMockIpcResponse<AppUpgradeEvent>({
    channel: 'app-upgradeEvent',
    response: {
      type: 'APP_UPGRADE_STATUS_STARTED_INSTALLER',
    },
  });

  const installUpdateButton = page.getByRole('button', {
    name: 'Install update',
  });

  const appUpgradePromise = util.mockIpcHandle({
    channel: 'appUpgrade',
    response: undefined,
  });

  await installUpdateButton.click();

  // The appUpgrade promise is resolved when its handle has been called.
  // The handle should be called when the Install update button is clicked.
  await appUpgradePromise;
});
