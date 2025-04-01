import { expect, test } from '@playwright/test';
import { Page } from 'playwright';

import { AppUpgradeEvent } from '../../../src/shared/app-upgrade';
import { IAppVersionInfo } from '../../../src/shared/daemon-rpc-types';
import { MockedTestUtils, startMockedApp } from './mocked-utils';

let page: Page;
let util: MockedTestUtils;

test.beforeAll(async () => {
  ({ page, util } = await startMockedApp());

  // void util.mockIpcHandle<undefined>({
  //   channel: 'app-upgrade',
  //   response: undefined,
  // });

  // void util.mockIpcHandle<undefined>({
  //   channel: 'app-upgrade-install',
  //   response: undefined,
  // });

  await util.sendMockIpcResponse<IAppVersionInfo>({
    channel: 'upgradeVersion-',
    response: {
      supported: true,
      suggestedIsBeta: false,
      suggestedUpgrade: {
        version: '2100.1',
        changelog:
          'This is a changelog.\nEach item is separated by a newline.\nThere are three items.',
      },
    },
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
    'This is a changelog.Each item is separated by a newline.There are three items.',
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

  // The cancel button should be hidden when the upgrade is canceled
  await expect(cancelButton).toBeHidden();

  // The Download & install button should become visible again
  const downloadAndInstallButton = page.getByRole('button', {
    name: 'Download and install',
  });
  await expect(downloadAndInstallButton).toBeVisible();

  await new Promise((resolve) => {});
});
