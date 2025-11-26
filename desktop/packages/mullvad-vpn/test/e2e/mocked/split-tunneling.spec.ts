import { expect, test } from '@playwright/test';
import { Page } from 'playwright';

import { ISplitTunnelingApplication } from '../../../src/shared/application-types';
import { RoutePath } from '../../../src/shared/routes';
import { RoutesObjectModel } from '../route-object-models';
import { MockedTestUtils, startMockedApp } from './mocked-utils';

let page: Page;
let util: MockedTestUtils;
let routes: RoutesObjectModel;

const applications: Array<ISplitTunnelingApplication> = [
  { name: 'microsoft edge', absolutepath: '/path/to/program/msedge.exe', deletable: false },
  {
    name: 'windows media player legacy',
    absolutepath: '/path/to/program/wmplayer.exe',
    deletable: false,
  },
  {
    name: 'mission control',
    absolutepath: '/path/to/program/missioncontrol.app',
    deletable: false,
  },
];

const startup = async (postLaunch?: () => Promise<void>) => {
  ({ page, util } = await startMockedApp());
  routes = new RoutesObjectModel(page, util);

  await util.expectRoute(RoutePath.main);

  await postLaunch?.();

  await routes.main.gotoSettings();
  await routes.settings.gotoSplitTunnelingSettings();
};

test.describe('Windows and macOS split tunneling', () => {
  if (process.platform === 'linux') {
    test.skip();
  }

  test.afterAll(async () => {
    await util?.closePage();
  });

  test.beforeAll(async () => {
    await startup(async () => {
      await util.ipc.macOsSplitTunneling.needFullDiskPermissions.handle(false);
      await util.ipc.splitTunneling.getApplications.handle({ fromCache: false, applications });
    });
  });

  test('List of split applications should not be visible', async () => {
    const splitList = routes.splitTunnelingSettings.selectors.splitApplicationsList();
    const nonSplitList = routes.splitTunnelingSettings.selectors.nonSplitApplicationsList();
    await expect(splitList).not.toBeVisible();
    await expect(nonSplitList).toBeVisible();
  });

  applications.forEach((application, index, applications) => {
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

      await Promise.all([
        util.ipc.splitTunneling.addApplication.expect(),
        routes.splitTunnelingSettings.toggleApplication(nonSplitApplication),
      ]);
      await util.ipc.splitTunneling[''].notify(applications.slice(0, index + 1));

      await expect(splitApplication).toBeVisible();
      await expect(nonSplitApplication).not.toBeVisible();

      await expect(
        routes.splitTunnelingSettings.selectors.applicationButtonsInList(splitList),
      ).toHaveCount(index + 1);
    });
  });

  test('List of split applications should be visible', async () => {
    const splitList = routes.splitTunnelingSettings.selectors.splitApplicationsList();
    const nonSplitList = routes.splitTunnelingSettings.selectors.nonSplitApplicationsList();
    await expect(splitList).toBeVisible();
    await expect(nonSplitList).not.toBeVisible();
  });

  applications.forEach((application, index, applications) => {
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

      await Promise.all([
        util.ipc.splitTunneling.removeApplication.expect(),
        routes.splitTunnelingSettings.toggleApplication(splitApplication),
      ]);
      await util.ipc.splitTunneling[''].notify(applications.slice(index + 1));

      await expect(splitApplication).not.toBeVisible();
      await expect(nonSplitApplication).toBeVisible();

      await expect(
        routes.splitTunnelingSettings.selectors.applicationButtonsInList(splitList),
      ).toHaveCount(applications.length - index - 1);
    });
  });
});
