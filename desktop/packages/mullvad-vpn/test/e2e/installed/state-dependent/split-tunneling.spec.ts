import { expect, test } from '@playwright/test';
import { execSync } from 'child_process';
import { Page } from 'playwright';

import { RoutePath } from '../../../../src/shared/routes';
import { RoutesObjectModel } from '../../route-object-models';
import { TestUtils } from '../../utils';
import { startInstalledApp } from '../installed-utils';

// Windows and macOS only. This test expects the daemon to be logged in and for split tunneling to
// be off and have no split applications.

const applications =
  process.platform === 'win32'
    ? ['microsoft edge', 'windows media player legacy']
    : ['launchpad', 'clock'];

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
    await expect(title).toHaveText('Split tunneling');

    const toggle = routes.splitTunnelingSettings.selectors.splitTunnelingSwitch();
    await expect(toggle).not.toBeChecked();

    const splitList = routes.splitTunnelingSettings.selectors.splitApplicationsList();
    const nonSplitList = routes.splitTunnelingSettings.selectors.nonSplitApplicationsList();

    await expect(splitList).not.toBeVisible();
    await expect(nonSplitList).not.toBeVisible();

    const application = routes.splitTunnelingSettings.selectors.application(applications[0]);
    await expect(application).not.toBeVisible();

    await routes.splitTunnelingSettings.toggleSplitTunneling();
    await expect(toggle).toBeChecked();
    await expect(splitList).not.toBeVisible();
    await expect(nonSplitList).toBeVisible();
    await expect(application).toBeVisible();

    const numberOfSplitApplications =
      await routes.splitTunnelingSettings.numberOfSplitApplications();
    expect(numberOfSplitApplications).toBe(0);
    expect(getDaemonSplitTunnelingApplications()).toHaveLength(0);
  });

  applications.forEach((application, index, applications) => {
    test(`App should split ${application}`, async () => {
      const splitList = routes.splitTunnelingSettings.selectors.splitApplicationsList();
      const nonSplitList = routes.splitTunnelingSettings.selectors.nonSplitApplicationsList();

      const splitApplication = routes.splitTunnelingSettings.selectors.applicationInList(
        splitList,
        application,
      );
      const nonSplitApplication = routes.splitTunnelingSettings.selectors.applicationInList(
        nonSplitList,
        application,
      );

      await expect(splitApplication).not.toBeVisible();
      await expect(nonSplitApplication).toBeVisible();

      await routes.splitTunnelingSettings.toggleApplication(nonSplitApplication);

      await expect(splitApplication).toBeVisible();
      await expect(nonSplitApplication).not.toBeVisible();

      const numberOfSplitApplications =
        await routes.splitTunnelingSettings.numberOfSplitApplications();
      expect(numberOfSplitApplications).toBe(index + 1);

      const daemonSplitTunnelingApplications = getDaemonSplitTunnelingApplications();
      expect(daemonSplitTunnelingApplications).toHaveLength(index + 1);

      applications.slice(0, index + 1).forEach((application) => {
        expect(isSplitInDaemon(application)).toBeTruthy();
      });
    });
  });

  test(`App should unsplit ${applications[0]}`, async () => {
    const application = applications[0];
    const splitList = routes.splitTunnelingSettings.selectors.splitApplicationsList();
    const nonSplitList = routes.splitTunnelingSettings.selectors.nonSplitApplicationsList();

    const splitApplication = routes.splitTunnelingSettings.selectors.applicationInList(
      splitList,
      application,
    );
    const nonSplitApplication = routes.splitTunnelingSettings.selectors.applicationInList(
      nonSplitList,
      application,
    );

    await expect(splitApplication).toBeVisible();
    await expect(nonSplitApplication).not.toBeVisible();

    await routes.splitTunnelingSettings.toggleApplication(splitApplication);

    await expect(splitApplication).not.toBeVisible();
    await expect(nonSplitApplication).toBeVisible();

    const numberOfSplitApplications =
      await routes.splitTunnelingSettings.numberOfSplitApplications();
    expect(numberOfSplitApplications).toBe(1);

    const daemonSplitTunnelingApplications = getDaemonSplitTunnelingApplications();
    expect(daemonSplitTunnelingApplications).toHaveLength(1);
    expect(isSplitInDaemon(application)).toBeFalsy();
    expect(isSplitInDaemon(applications[1])).toBeTruthy();
  });

  test('App should disable split tunneling', async () => {
    const toggle = routes.splitTunnelingSettings.selectors.splitTunnelingSwitch();
    await expect(toggle).toBeChecked();

    const splitList = routes.splitTunnelingSettings.selectors.splitApplicationsList();
    const nonSplitList = routes.splitTunnelingSettings.selectors.nonSplitApplicationsList();

    await expect(splitList).toBeVisible();
    await expect(nonSplitList).toBeVisible();

    const application = routes.splitTunnelingSettings.selectors.application(applications[0]);
    await expect(application).toBeVisible();

    await routes.splitTunnelingSettings.toggleSplitTunneling();
    await expect(toggle).not.toBeChecked();
  });
});

function getDaemonSplitTunnelingApplications() {
  const output = execSync('mullvad split-tunnel get').toString().trim().split('\n');
  return output.slice(output.indexOf('Excluded applications:') + 1);
}

function isSplitInDaemon(app: string): boolean {
  return !!getDaemonSplitTunnelingApplications().find((splitApp) =>
    splitApp.toLowerCase().includes(app),
  );
}
