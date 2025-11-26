import { expect, test } from '@playwright/test';
import { execSync } from 'child_process';
import { Page } from 'playwright';

import { RoutePath } from '../../../../src/shared/routes';
import { RoutesObjectModel } from '../../route-object-models';
import { TestUtils } from '../../utils';
import { startInstalledApp } from '../installed-utils';

// Windows and macOS only. This test expects the daemon to be logged in and for split tunneling to
// be off and have no split applications.

interface Application {
  name: string;
  filename?: string;
}

const application: Application =
  process.platform === 'win32'
    ? { name: 'microsoft edge', filename: 'msedge.exe' }
    : { name: 'launchpad' };

let page: Page;
let util: TestUtils;
let routes: RoutesObjectModel;

test.describe('Split tunneling', () => {
  test.beforeAll(async () => {
    ({ page, util } = await startInstalledApp());
    routes = new RoutesObjectModel(page, util);

    await util.expectRoute(RoutePath.main);

    await routes.main.gotoSettings();
    await routes.settings.gotoSplitTunnelingSettings();
  });

  test.afterAll(async () => {
    await util?.closePage();
  });

  test('App should enable split tunneling', async () => {
    const title = routes.splitTunnelingSettings.selectors.heading();
    await expect(title).toBeVisible();

    const toggle = routes.splitTunnelingSettings.selectors.splitTunnelingSwitch();
    await expect(toggle).not.toBeChecked();

    const splitList = routes.splitTunnelingSettings.selectors.splitApplicationsList();
    const nonSplitList = routes.splitTunnelingSettings.selectors.nonSplitApplicationsList();

    await expect(splitList).not.toBeVisible();
    await expect(nonSplitList).not.toBeVisible();

    const applicationLocator = routes.splitTunnelingSettings.selectors.application(
      application.name,
    );
    await expect(applicationLocator).not.toBeVisible();

    await routes.splitTunnelingSettings.toggleSplitTunneling();
    await expect(toggle).toBeChecked();
    await expect(splitList).not.toBeVisible();
    await expect(nonSplitList).toBeVisible();
    await expect(applicationLocator).toBeVisible();
  });

  test(`App should split ${application.name}`, async () => {
    const splitList = routes.splitTunnelingSettings.selectors.splitApplicationsList();
    const nonSplitList = routes.splitTunnelingSettings.selectors.nonSplitApplicationsList();

    const splitApplication = routes.splitTunnelingSettings.selectors.applicationInList(
      splitList,
      application.name,
    );
    const nonSplitApplication = routes.splitTunnelingSettings.selectors.applicationInList(
      nonSplitList,
      application.name,
    );

    await expect(splitApplication).not.toBeVisible();
    await expect(nonSplitApplication).toBeVisible();

    await routes.splitTunnelingSettings.toggleApplication(nonSplitApplication);

    await expect(splitList).toBeVisible();
    await expect(splitApplication).toBeVisible();
    await expect(nonSplitApplication).not.toBeVisible();

    await expect(
      routes.splitTunnelingSettings.selectors.applicationButtonsInList(splitList),
    ).toHaveCount(1);

    const daemonSplitTunnelingApplications = getDaemonSplitTunnelingApplications();
    expect(daemonSplitTunnelingApplications).toHaveLength(1);
    expect(isSplitInDaemon(application)).toBeTruthy();
  });

  test('App should disable split tunneling', async () => {
    const toggle = routes.splitTunnelingSettings.selectors.splitTunnelingSwitch();
    await expect(toggle).toBeChecked();

    const splitList = routes.splitTunnelingSettings.selectors.splitApplicationsList();
    const nonSplitList = routes.splitTunnelingSettings.selectors.nonSplitApplicationsList();

    await expect(splitList).toBeVisible();

    const applicationLocator = routes.splitTunnelingSettings.selectors.application(
      application.name,
    );
    await expect(applicationLocator).toBeVisible();

    await routes.splitTunnelingSettings.toggleSplitTunneling();
    await expect(toggle).not.toBeChecked();

    await expect(applicationLocator).not.toBeVisible();
    await expect(splitList).not.toBeVisible();
    await expect(nonSplitList).not.toBeVisible();
  });

  test('App should reenable split tunneling', async () => {
    const toggle = routes.splitTunnelingSettings.selectors.splitTunnelingSwitch();
    await expect(toggle).not.toBeChecked();

    const splitList = routes.splitTunnelingSettings.selectors.splitApplicationsList();
    const nonSplitList = routes.splitTunnelingSettings.selectors.nonSplitApplicationsList();
    const applicationLocator = routes.splitTunnelingSettings.selectors.applicationInList(
      splitList,
      application.name,
    );

    await expect(splitList).not.toBeVisible();
    await expect(nonSplitList).not.toBeVisible();
    await expect(applicationLocator).not.toBeVisible();

    await routes.splitTunnelingSettings.toggleSplitTunneling();
    await expect(toggle).toBeChecked();

    await expect(splitList).toBeVisible();
    await expect(applicationLocator).toBeVisible();

    await expect(
      routes.splitTunnelingSettings.selectors.applicationButtonsInList(splitList),
    ).toHaveCount(1);
  });

  test(`App should unsplit ${application.name}`, async () => {
    const splitList = routes.splitTunnelingSettings.selectors.splitApplicationsList();
    const nonSplitList = routes.splitTunnelingSettings.selectors.nonSplitApplicationsList();

    const splitApplication = routes.splitTunnelingSettings.selectors.applicationInList(
      splitList,
      application.name,
    );
    const nonSplitApplication = routes.splitTunnelingSettings.selectors.applicationInList(
      nonSplitList,
      application.name,
    );

    await expect(splitApplication).toBeVisible();
    await expect(nonSplitApplication).not.toBeVisible();

    await routes.splitTunnelingSettings.toggleApplication(splitApplication);

    await expect(splitApplication).not.toBeVisible();
    await expect(nonSplitApplication).toBeVisible();

    await expect(splitList).not.toBeVisible();
    await expect(
      routes.splitTunnelingSettings.selectors.applicationButtonsInList(splitList),
    ).toHaveCount(0);

    const daemonSplitTunnelingApplications = getDaemonSplitTunnelingApplications();
    expect(daemonSplitTunnelingApplications).toHaveLength(0);
    expect(isSplitInDaemon(application)).toBeFalsy();
  });
});

function getDaemonSplitTunnelingApplications() {
  const output = execSync('mullvad split-tunnel get').toString().trim().split('\n');
  return output.slice(output.indexOf('Excluded applications:') + 1);
}

function isSplitInDaemon(application: Application): boolean {
  return !!getDaemonSplitTunnelingApplications().find((splitApp) =>
    splitApp.toLowerCase().includes(application.filename ?? application.name),
  );
}
