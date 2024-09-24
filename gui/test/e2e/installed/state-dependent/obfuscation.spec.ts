import { expect, test } from '@playwright/test';
import { execSync } from 'child_process';
import { Page } from 'playwright';

import { colors } from '../../../../src/config.json';
import { RoutePath } from '../../../../src/renderer/lib/routes';
import { TestUtils } from '../../utils';
import { startInstalledApp } from '../installed-utils';

const SHADOWSOCKS_PORT = 65_000;
const UDPOVERTCP_PORT = '80';

// This test sets different obfuscation settings combinations and verifies that it was set in the
// daemon.

let page: Page;
let util: TestUtils;

test.beforeAll(async () => {
  ({ page, util } = await startInstalledApp());
});

test.afterAll(async () => {
  await page.close();
});

test('App should have automatic obfuscation', async () => {
  await util.waitForNavigation(() => page.click('button[aria-label="Settings"]'));
  expect(await util.waitForNavigation(() => page.getByText('VPN settings').click())).toBe(
    RoutePath.vpnSettings,
  );

  expect(await util.waitForNavigation(() => page.getByText('WireGuard settings').click())).toBe(
    RoutePath.wireguardSettings,
  );

  const automatic = page.getByTestId('automatic-obfuscation');
  await expect(automatic).toHaveCSS('background-color', colors.green);

  const cliObfuscation = execSync('mullvad obfuscation get').toString().split('\n');
  expect(cliObfuscation[0]).toEqual('Obfuscation mode: auto');
  expect(cliObfuscation[1]).toEqual('udp2tcp settings: any port');
  expect(cliObfuscation[2]).toEqual('Shadowsocks settings: any port');
});

test('App should set obfuscation to shadowsocks with custom port', async () => {
  expect(
    await util.waitForNavigation(() => page.click('button[aria-label="Shadowsocks settings"]')),
  ).toBe(RoutePath.shadowsocks);

  const automatic = page.locator('button', { hasText: 'Automatic' });
  await expect(automatic).toHaveCSS('background-color', colors.green);

  const customInput = page.locator('input[type="text"]');
  await customInput.click();
  await customInput.fill(`${SHADOWSOCKS_PORT}`);
  await customInput.blur();

  const customItem = page.locator('div[role="option"]', { hasText: 'Custom' });
  await expect(customItem).toHaveCSS('background-color', colors.green);

  await util.waitForNavigation(() => page.click('button[aria-label="Back"]'));

  const shadowsocksItem = page.locator('button', { hasText: 'Shadowsocks' });
  await shadowsocksItem.click();
  await expect(shadowsocksItem).toHaveCSS('background-color', colors.green);
  await expect(shadowsocksItem).toContainText(`Port: ${SHADOWSOCKS_PORT}`);

  const cliObfuscation = execSync('mullvad obfuscation get').toString().split('\n')[2];
  expect(cliObfuscation).toEqual(`Shadowsocks settings: port ${SHADOWSOCKS_PORT}`);
});

test('App should still have shadowsocks custom port', async () => {
  expect(
    await util.waitForNavigation(() => page.click('button[aria-label="Shadowsocks settings"]')),
  ).toBe(RoutePath.shadowsocks);

  const customItem = page.locator('div[role="option"]', { hasText: 'Custom' });
  await expect(customItem).toHaveCSS('background-color', colors.green);

  await util.waitForNavigation(() => page.click('button[aria-label="Back"]'));
});

test('App should set obfuscation to UDP-over-TCP with port', async () => {
  expect(
    await util.waitForNavigation(() => page.click('button[aria-label="UDP-over-TCP settings"]')),
  ).toBe(RoutePath.udpOverTcp);

  const automatic = page.locator('button', { hasText: 'Automatic' });
  await expect(automatic).toHaveCSS('background-color', colors.green);

  const portButton = page.locator('button', { hasText: UDPOVERTCP_PORT });
  await portButton.click();

  await expect(portButton).toHaveCSS('background-color', colors.green);

  await util.waitForNavigation(() => page.click('button[aria-label="Back"]'));

  const udpOverTcpItem = page.locator('button', { hasText: 'UDP-over-TCP' });
  await udpOverTcpItem.click();
  await expect(udpOverTcpItem).toHaveCSS('background-color', colors.green);
  await expect(udpOverTcpItem).toContainText(`Port: ${UDPOVERTCP_PORT}`);

  const cliObfuscation = execSync('mullvad obfuscation get').toString().split('\n')[1];
  expect(cliObfuscation).toEqual(`udp2tcp settings: port ${UDPOVERTCP_PORT}`);
});
