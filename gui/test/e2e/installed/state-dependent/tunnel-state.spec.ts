import { exec as execAsync } from 'child_process';
import { promisify } from 'util';
import { expect, test } from '@playwright/test';
import { Page } from 'playwright';
import {
  assertConnected,
  assertConnectedPq,
  assertConnecting,
  assertConnectingPq,
  assertDisconnected,
  assertDisconnecting,
  assertError,
} from '../../shared/tunnel-state';

import { startInstalledApp } from '../installed-utils';
import { escapeRegExp } from '../../utils';

const exec = promisify(execAsync);

// This test expects the daemon to be logged into an account that has time left and to be
// disconnected. Env parameters:
// HOSTNAME: hostname of the currently selected WireGuard relay
// IN_IP: In ip of the relay passed in `HOSTNAME`
// CONNECTION_CHECK_URL: Url to the connection check

let page: Page;

test.beforeAll(async () => {
  ({ page } = await startInstalledApp());
});

test.afterAll(async () => {
  await page.close();
});

test('App should show disconnected tunnel state', async () => {
  await assertDisconnected(page);
});

test('App should connect', async () => {
  await page.getByText('Secure my connection').click();

  await assertConnecting(page);
  await assertConnected(page);

  const relay = page.getByTestId('hostname-line');
  const inIp = page.locator(':text("In") + span');
  const outIp = page.locator(':text("Out") + div > span');

  await expect(relay).toHaveText(process.env.HOSTNAME!);
  await expect(inIp).not.toBeVisible();
  await relay.click();

  await expect(inIp).toBeVisible();
  expect(await inIp.textContent()).toMatch(new RegExp(`^${process.env.IN_IP!}`));

  await expect(outIp).toBeVisible();

  const ipResponse = await fetch(`${process.env.CONNECTION_CHECK_URL!}/ip`);
  const ip = await ipResponse.text();

  expect(await outIp.textContent()).toBe(ip.trim());
});

test('App should show correct WireGuard port', async () => {
  const inData = page.locator(':text("In") + span');

  await expect(inData).toContainText(new RegExp(':[0-9]+'));

  await exec('mullvad relay set tunnel wireguard --port=53');
  await assertConnected(page);
  await expect(inData).toContainText(new RegExp(':53'));

  await exec('mullvad relay set tunnel wireguard --port=51820');
  await assertConnected(page);
  await expect(inData).toContainText(new RegExp(':51820'));

  await exec('mullvad relay set tunnel wireguard --port=any');
});

test('App should show correct WireGuard transport protocol', async () => {
  const inData = page.locator(':text("In") + span');

  await exec('mullvad obfuscation set mode udp2tcp');
  await expect(inData).toContainText(new RegExp('TCP'));

  await exec('mullvad obfuscation set mode off');
  await expect(inData).toContainText(new RegExp('UDP$'));
});

test('App should show correct tunnel protocol', async () => {
  const tunnelProtocol = page.getByTestId('tunnel-protocol');
  await expect(tunnelProtocol).toHaveText('WireGuard');

  await exec('mullvad relay set tunnel-protocol openvpn');
  await exec('mullvad relay set location se');
  await assertConnected(page);
  await expect(tunnelProtocol).toHaveText('OpenVPN');
});

test('App should show correct OpenVPN transport protocol and port', async () => {
  const inData = page.locator(':text("In") + span');

  await expect(inData).toContainText(new RegExp(':[0-9]+'));
  await expect(inData).toContainText(new RegExp('(TCP|UDP)$'));
  await exec('mullvad relay set tunnel openvpn --transport-protocol udp --port 1195');
  await assertConnected(page);
  await expect(inData).toContainText(new RegExp(':1195'));

  await exec('mullvad relay set tunnel openvpn --transport-protocol udp --port 1300');
  await assertConnected(page);
  await expect(inData).toContainText(new RegExp(':1300'));

  await exec('mullvad relay set tunnel openvpn --transport-protocol tcp --port any');
  await expect(inData).toContainText(new RegExp(':[0-9]+'));
  await expect(inData).toContainText(new RegExp('TCP$'));

  await exec('mullvad relay set tunnel openvpn --transport-protocol tcp --port 80');
  await assertConnected(page);
  await expect(inData).toContainText(new RegExp(':80'));

  await exec('mullvad relay set tunnel openvpn --transport-protocol tcp --port 443');
  await assertConnected(page);
  await expect(inData).toContainText(new RegExp(':443'));

  await exec('mullvad relay set tunnel openvpn --transport-protocol any');
});

test('App should show bridge mode', async () => {
  await exec('mullvad bridge set state on');
  const relay = page.getByTestId('hostname-line');
  await expect(relay).toHaveText(new RegExp(' via ', 'i'));
  await exec('mullvad bridge set state off');

  await exec('mullvad relay set tunnel-protocol wireguard');
});

test('App should enter blocked state', async () => {
  await exec('mullvad relay set location xx');
  await assertError(page);

  await exec(`mullvad relay set hostname ${process.env.HOSTNAME}`);
  await assertConnected(page);
});

test('App should disconnect', async () => {
  await page.getByText('Disconnect').click();
  await assertDisconnected(page);
});

test('App should create quantum secure connection', async () => {
  await exec('mullvad tunnel set wireguard --quantum-resistant on');
  await page.getByText('Secure my connection').click();

  await assertConnectingPq(page);
  await assertConnectedPq(page);
});

test('App should show multihop', async () => {
  await exec('mullvad relay set tunnel wireguard --use-multihop=on');
  const relay = page.getByTestId('hostname-line');
  await expect(relay).toHaveText(new RegExp('^' + escapeRegExp(`${process.env.HOSTNAME} via`), 'i'));
  await exec('mullvad relay set tunnel wireguard --use-multihop=off');

  await exec('mullvad tunnel set wireguard --quantum-resistant off');
  await page.getByText('Disconnect').click();
});


test('App should become connected when other frontend connects', async () => {
  await assertDisconnected(page);
  await Promise.all([assertConnecting(page), exec('mullvad connect')]);
  await assertConnected(page);

  await Promise.all([assertDisconnecting(page), exec('mullvad disconnect')]);
  await assertDisconnected(page);
});
