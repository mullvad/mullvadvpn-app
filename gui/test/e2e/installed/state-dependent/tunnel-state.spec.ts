import { exec as execAsync } from 'child_process';
import { promisify } from 'util';
import { expect, test } from '@playwright/test';
import { Page } from 'playwright';
import {
  expectConnected,
  expectConnectedPq,
  expectDisconnected,
  expectError,
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
  await expectDisconnected(page);
});

test('App should connect', async () => {
  await page.getByText('Secure my connection').click();
  await expectConnected(page);

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
  const inData = page.getByTestId('in-ip');

  await expect(inData).toContainText(new RegExp(':[0-9]+'));

  await exec('mullvad relay set tunnel wireguard --port=53');
  await expectConnected(page);
  await expect(inData).toContainText(new RegExp(':53'));

  await exec('mullvad relay set tunnel wireguard --port=51820');
  await expectConnected(page);
  await expect(inData).toContainText(new RegExp(':51820'));

  await exec('mullvad relay set tunnel wireguard --port=any');
});

test('App should show correct WireGuard transport protocol', async () => {
  const inData = page.getByTestId('in-ip');

  await exec('mullvad obfuscation set mode udp2tcp');
  await expectConnected(page);
  await expect(inData).toContainText(new RegExp('TCP'));

  await exec('mullvad obfuscation set mode off');
  await expectConnected(page);
  await expect(inData).toContainText(new RegExp('UDP$'));
});

test('App should show correct tunnel protocol', async () => {
  const tunnelProtocol = page.getByTestId('tunnel-protocol');
  await expect(tunnelProtocol).toHaveText('WireGuard');

  await exec('mullvad relay set tunnel-protocol openvpn');
  await exec('mullvad relay set location se');
  await expectConnected(page);
  await expect(tunnelProtocol).toHaveText('OpenVPN');
});

test('App should show correct OpenVPN transport protocol and port', async () => {
  const inData = page.getByTestId('in-ip');

  await expect(inData).toContainText(new RegExp(':[0-9]+'));
  await expect(inData).toContainText(new RegExp('(TCP|UDP)$'));
  await exec('mullvad relay set tunnel openvpn --transport-protocol udp --port 1195');
  await expectConnected(page);
  await expect(inData).toContainText(new RegExp(':1195'));

  await exec('mullvad relay set tunnel openvpn --transport-protocol udp --port 1300');
  await expectConnected(page);
  await expect(inData).toContainText(new RegExp(':1300'));

  await exec('mullvad relay set tunnel openvpn --transport-protocol tcp --port any');
  await expectConnected(page);
  await expect(inData).toContainText(new RegExp(':[0-9]+'));
  await expect(inData).toContainText(new RegExp('TCP$'));

  await exec('mullvad relay set tunnel openvpn --transport-protocol tcp --port 80');
  await expectConnected(page);
  await expect(inData).toContainText(new RegExp(':80'));

  await exec('mullvad relay set tunnel openvpn --transport-protocol tcp --port 443');
  await expectConnected(page);
  await expect(inData).toContainText(new RegExp(':443'));

  await exec('mullvad relay set tunnel openvpn --transport-protocol any');
});

test('App should show bridge mode', async () => {
  await exec('mullvad bridge set state on');
  await expectConnected(page);
  const relay = page.getByTestId('hostname-line');
  await expect(relay).toHaveText(new RegExp(' via ', 'i'));
  await exec('mullvad bridge set state off');

  await exec('mullvad relay set tunnel-protocol wireguard');
});

test('App should enter blocked state', async () => {
  await exec('mullvad debug block-connection');
  await expectError(page);

  await exec(`mullvad relay set location ${process.env.HOSTNAME}`);
  await expectConnected(page);
});

test('App should disconnect', async () => {
  await page.getByText('Disconnect').click();
  await expectDisconnected(page);
});

test('App should create quantum secure connection', async () => {
  await exec('mullvad tunnel set wireguard --quantum-resistant on');
  await page.getByText('Secure my connection').click();

  await expectConnectedPq(page);
});

test('App should show multihop', async () => {
  await exec('mullvad relay set tunnel wireguard --use-multihop=on');
  await expectConnectedPq(page);
  const relay = page.getByTestId('hostname-line');
  await expect(relay).toHaveText(new RegExp('^' + escapeRegExp(`${process.env.HOSTNAME} via`), 'i'));
  await exec('mullvad relay set tunnel wireguard --use-multihop=off');

  await exec('mullvad tunnel set wireguard --quantum-resistant off');
  await page.getByText('Disconnect').click();
});


test('App should become connected when other frontend connects', async () => {
  await expectDisconnected(page);
  await exec('mullvad connect');
  await expectConnected(page);

  await exec('mullvad disconnect');
  await expectDisconnected(page);
});
