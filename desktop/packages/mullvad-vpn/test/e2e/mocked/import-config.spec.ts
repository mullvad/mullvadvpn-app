import { expect, test } from '@playwright/test';
import { Page } from 'playwright';

import { RoutesObjectModel } from '../route-object-models';
import { MockedTestUtils, startMockedApp } from './mocked-utils';

let page: Page;
let util: MockedTestUtils;
let routes: RoutesObjectModel;

test.beforeAll(async () => {
  ({ page, util } = await startMockedApp());
  routes = new RoutesObjectModel(page, util);
  await routes.main.waitForRoute();
});

test.afterAll(async () => {
  await util?.closePage();
});

test('The import view is shown when there is no WireGuard config', async () => {
  await util.ipc.identity[''].notify(undefined);
  await routes.importConfig.waitForRoute();

  await expect(routes.importConfig.selectors.configTextArea()).toBeVisible();
  await expect(routes.importConfig.selectors.importTextButton()).toBeDisabled();
});

test('An invalid config is rejected and keeps the import view open', async () => {
  await util.ipc.identity.importText.handle({
    type: 'error',
    error: 'invalid-config',
    message: 'The config has no PrivateKey',
  });

  await routes.importConfig.importPastedConfig('[Interface]\nAddress = 10.64.0.2/32\n');
  await expect(routes.importConfig.selectors.subtitle()).toContainText(
    /not a valid WireGuard config/i,
  );
  await routes.importConfig.waitForRoute();
});

test('Importing a valid config leads to the main view', async () => {
  const identity = {
    publicKey: 'bXlwdWJsaWNrZXlmb3J0ZXN0aW5nb25seTEyMzQ1Njc=',
    ipv4Address: '10.64.0.2/32',
    ipv6Address: 'fc00:bbbb:bbbb:bb01::1/128',
  };

  await util.ipc.identity.importText.handle({ type: 'success', identity });

  await routes.importConfig.importPastedConfig('[Interface]\nPrivateKey = x\nAddress = y\n');
  // The daemon reports the new identity through an event, which makes the app navigate.
  await util.ipc.identity[''].notify(identity);
  await routes.main.waitForRoute();
});
