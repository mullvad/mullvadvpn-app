import { expect, Locator, test } from '@playwright/test';
import { execSync } from 'child_process';
import { Page } from 'playwright';

import { RoutePath } from '../../../../src/renderer/lib/routes';
import { TestUtils } from '../../utils';
import { startInstalledApp } from '../installed-utils';

// macOS only. This test expects the daemon to be logged in and for split tunneling to be off and
// have no split applications.

let page: Page;
let util: TestUtils;

test.beforeAll(async () => {
  ({ page, util } = await startInstalledApp());
});

test.afterAll(async () => {
  await page.close();
});

async function navigateToSplitTunneling() {
  await page.click('button[aria-label="Settings"]');
  await util.waitForRoute(RoutePath.settings);

  await page.getByText('Split tunneling').click();
  await util.waitForRoute(RoutePath.splitTunneling);

  const title = page.locator('h1');
  await expect(title).toHaveText('Split tunneling');
}

test('App should enable split tunneling', async () => {
  await navigateToSplitTunneling();

  const toggle = page.getByRole('checkbox');
  await expect(toggle).not.toBeChecked();

  const splitList = page.getByTestId('split-applications');
  const nonSplitList = page.getByTestId('non-split-applications');

  await expect(splitList).not.toBeVisible();
  await expect(nonSplitList).not.toBeVisible();

  const launchPadApp = page.getByText('launchpad');
  await expect(launchPadApp).not.toBeVisible();

  await toggle.click();
  await expect(toggle).toBeChecked();
  await expect(splitList).not.toBeVisible();
  await expect(nonSplitList).toBeVisible();
  await expect(launchPadApp).toBeVisible();
  expect(await numberOfApplicationsInList('split-applications')).toBe(0);
  expect(getDaemonSplitTunnelingApplications()).toHaveLength(0);
});

test('App should split launchpad', async () => {
  const splitList = page.getByTestId('split-applications');
  const nonSplitList = page.getByTestId('non-split-applications');

  const splitLaunchPadApp = splitList.getByText('launchpad');
  const nonSplitLaunchPadApp = nonSplitList.getByText('launchpad');

  await expect(splitLaunchPadApp).not.toBeVisible();
  await expect(nonSplitLaunchPadApp).toBeVisible();

  await toggleApplication(nonSplitLaunchPadApp);

  await expect(splitLaunchPadApp).toBeVisible();
  await expect(nonSplitLaunchPadApp).not.toBeVisible();
  expect(await numberOfApplicationsInList('split-applications')).toBe(1);

  const daemonSplitTunnelingApplications = getDaemonSplitTunnelingApplications();
  expect(daemonSplitTunnelingApplications).toHaveLength(1);
  expect(isSplitInDaemon('launchpad')).toBeTruthy();
});

test('App should split clock', async () => {
  const splitList = page.getByTestId('split-applications');
  const nonSplitList = page.getByTestId('non-split-applications');

  const splitClockApp = splitList.getByText('clock');
  const nonSplitClockApp = nonSplitList.getByText('clock');

  await expect(splitClockApp).not.toBeVisible();
  await expect(nonSplitClockApp).toBeVisible();

  await toggleApplication(nonSplitClockApp);

  await expect(splitClockApp).toBeVisible();
  await expect(nonSplitClockApp).not.toBeVisible();
  expect(await numberOfApplicationsInList('split-applications')).toBe(2);

  const daemonSplitTunnelingApplications = getDaemonSplitTunnelingApplications();
  expect(daemonSplitTunnelingApplications).toHaveLength(2);
  expect(isSplitInDaemon('launchpad')).toBeTruthy();
  expect(isSplitInDaemon('clock')).toBeTruthy();
});

test('App should unsplit launchpad', async () => {
  const splitList = page.getByTestId('split-applications');
  const nonSplitList = page.getByTestId('non-split-applications');

  const splitLaunchPadApp = splitList.getByText('launchpad');
  const nonSplitLaunchPadApp = nonSplitList.getByText('launchpad');

  await expect(splitLaunchPadApp).toBeVisible();
  await expect(nonSplitLaunchPadApp).not.toBeVisible();

  await toggleApplication(splitLaunchPadApp);

  await expect(splitLaunchPadApp).not.toBeVisible();
  await expect(nonSplitLaunchPadApp).toBeVisible();
  expect(await numberOfApplicationsInList('split-applications')).toBe(1);

  const daemonSplitTunnelingApplications = getDaemonSplitTunnelingApplications();
  expect(daemonSplitTunnelingApplications).toHaveLength(1);
  expect(isSplitInDaemon('launchpad')).toBeFalsy();
  expect(isSplitInDaemon('clock')).toBeTruthy();
});

test('App should disable split tunneling', async () => {
  const toggle = page.getByRole('checkbox');
  await expect(toggle).toBeChecked();

  const splitList = page.getByTestId('split-applications');
  const nonSplitList = page.getByTestId('non-split-applications');

  await expect(splitList).toBeVisible();
  await expect(nonSplitList).toBeVisible();

  const launchPadApp = page.getByText('launchpad');
  await expect(launchPadApp).toBeVisible();

  await toggle.click();
  await expect(toggle).not.toBeChecked();
});

async function toggleApplication(applicationLocator: Locator) {
  await applicationLocator.locator('~ div').click();
}

async function numberOfApplicationsInList(listTestid: string) {
  const list = page.getByTestId(listTestid);
  const listHidden = await list.isHidden();
  if (listHidden) {
    return 0;
  }

  return list.locator('button').count();
}

function getDaemonSplitTunnelingApplications() {
  const output = execSync('mullvad split-tunnel get').toString().trim().split('\n');
  return output.slice(output.indexOf('Excluded applications:') + 1);
}

function isSplitInDaemon(app: string): boolean {
  return !!getDaemonSplitTunnelingApplications().find((splitApp) =>
    splitApp.toLowerCase().includes(app),
  );
}
