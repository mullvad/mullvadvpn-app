import { test } from '@playwright/test';
import { Page } from 'playwright';

import { RoutePath } from '../../../../src/shared/routes';
import { expectDisconnected } from '../../shared/tunnel-state';
import { TestUtils } from '../../utils';
import { startInstalledApp } from '../installed-utils';

// This test isn't called anywhere and is available as a simple test to run manually to verify that
// running tests works as expected. This is useful e.g. when updating packages such as electron,
// electron builder, playwright or when making changes to build scripts.
//
// This test expects the daemon to be logged into an account that has time left and to be
// disconnected.

let page: Page;
let util: TestUtils;

test.beforeAll(async () => {
  ({ page, util } = await startInstalledApp());
  await util.expectRoute(RoutePath.main);
});

test.afterAll(async () => {
  await util?.closePage();
});

test('App should show disconnected tunnel state', async () => {
  await expectDisconnected(page);
});
