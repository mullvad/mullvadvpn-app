import { expect, test } from '@playwright/test';
import { exec as execAsync } from 'child_process';
import { Page } from 'playwright';
import { promisify } from 'util';

import { RoutePath } from '../../../../src/renderer/lib/routes';
import { expectConnected } from '../../shared/tunnel-state';
import { TestUtils } from '../../utils';
import { startInstalledApp } from '../installed-utils';

const exec = promisify(execAsync);

// This test expects the daemon to be logged into an account that has time left, have OpenVPN
// selected and to be disconnected.

let page: Page;
let util: TestUtils;

test.beforeAll(async () => {
  ({ page, util } = await startInstalledApp());
  await util.waitForRoute(RoutePath.main);
});

test.afterAll(async () => {
  await page.close();
});

test('App should show correct tunnel protocol', async () => {
  await page.getByText('Connect', { exact: true }).click();
  await expectConnected(page);
  await page.getByTestId('connection-panel-chevron').click();

  const tunnelProtocol = page.getByTestId('tunnel-protocol');
  await expect(tunnelProtocol).toHaveText('OpenVPN');
});

test('App should show correct OpenVPN transport protocol and port', async () => {
  const inData = page.getByTestId('in-ip');

  await expect(inData).toContainText(new RegExp(':[0-9]+'));
  await expect(inData).toContainText(new RegExp('(TCP|UDP)$'));

  await exec('mullvad relay set tunnel openvpn --transport-protocol udp --port 1300');
  await expectConnected(page);
  await page.getByTestId('connection-panel-chevron').click();
  await expect(inData).toContainText(new RegExp(':1300'));

  await exec('mullvad relay set tunnel openvpn --transport-protocol tcp --port any');
  await expectConnected(page);
  await page.getByTestId('connection-panel-chevron').click();
  await expect(inData).toContainText(new RegExp(':[0-9]+'));
  await expect(inData).toContainText(new RegExp('TCP$'));

  await exec('mullvad relay set tunnel openvpn --transport-protocol tcp --port 80');
  await expectConnected(page);
  await page.getByTestId('connection-panel-chevron').click();
  await expect(inData).toContainText(new RegExp(':80'));

  await exec('mullvad relay set tunnel openvpn --transport-protocol any');
});

test('App should show bridge mode', async () => {
  await exec('mullvad bridge set state on');
  await expectConnected(page);
  const relay = page.getByTestId('hostname-line');
  await expect(relay).toHaveText(new RegExp(' via ', 'i'));
});
