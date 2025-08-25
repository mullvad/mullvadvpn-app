import { expect, test } from '@playwright/test';
import { exec as execAsync } from 'child_process';
import { Page } from 'playwright';
import { promisify } from 'util';

import { RoutesObjectModel } from '../../route-object-models';
import { expectConnected, expectDisconnected, expectError } from '../../shared/tunnel-state';
import { escapeRegExp, TestUtils } from '../../utils';
import { startInstalledApp } from '../installed-utils';

const exec = promisify(execAsync);

// This test expects the daemon to be logged into an account that has time left and to be
// disconnected. Env parameters:
// HOSTNAME: hostname of the currently selected WireGuard relay
// IN_IP: In ip of the relay passed in `HOSTNAME`
// CONNECTION_CHECK_URL: Url to the connection check

const { HOSTNAME, IN_IP, CONNECTION_CHECK_URL } = process.env;

let page: Page;
let util: TestUtils;
let routes: RoutesObjectModel;

test.describe('Tunnel state and settings', () => {
  const startup = async () => {
    ({ page, util } = await startInstalledApp());
    routes = new RoutesObjectModel(page, util);

    await routes.main.waitForRoute();
  };

  test.beforeAll(async () => {
    await startup();
  });

  test.afterAll(async () => {
    await page.close();
  });

  test('App should show disconnected tunnel state', async () => {
    await expectDisconnected(page);
  });

  test('App should connect', async () => {
    await page.getByText('Connect', { exact: true }).click();
    await expectConnected(page);

    const relay = page.getByTestId('hostname-line');
    const inIp = page.getByTestId('in-ip');
    // If IPv6 is enabled, there will be two "Out" IPs, one for IPv4 and one for IPv6
    // Selecting the first resolves to the IPv4 address regardless of the IP setting
    const outIp = page.locator(':text("Out") + div > span').first();

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

    await exec('mullvad obfuscation set mode off');
    await exec('mullvad relay set tunnel wireguard --port=53');
    await expectConnected(page);
    await page.getByTestId('connection-panel-chevron').click();
    await expect(inData).toContainText(new RegExp(':53'));

    await exec('mullvad relay set tunnel wireguard --port=51820');
    await expectConnected(page);
    await page.getByTestId('connection-panel-chevron').click();
    await expect(inData).toContainText(new RegExp(':51820'));

    await exec('mullvad relay set tunnel wireguard --port=any');
    await exec('mullvad obfuscation set mode auto');
  });

  test.describe('Wireguard UDP-over-TCP', () => {
    async function gotoWireguardSettings() {
      await routes.main.gotoSettings();
      await routes.settings.gotoVpnSettings();
      await routes.vpnSettings.gotoWireguardSettings();
    }

    async function gotoUdpOverTcpSettings() {
      await gotoWireguardSettings();
      await routes.wireguardSettings.gotoUdpOverTcpSettings();
    }

    test.afterAll(async () => {
      await routes.wireguardSettings.gotoRoot();
    });

    test('App should show UDP', async () => {
      await exec('mullvad connect --wait');
      await expectConnected(page);
      await routes.main.expandConnectionPanel();
      const inValue = await routes.main.getInValueText();
      expect(inValue).toMatch(new RegExp('UDP$'));
    });

    test('App should enable UDP-over-TCP', async () => {
      await gotoWireguardSettings();

      const udpOverTcpOption = routes.wireguardSettings.getUdpOverTcpOption();
      await expect(udpOverTcpOption).toHaveAttribute('aria-selected', 'false');

      await routes.wireguardSettings.selectUdpOverTcp();
      await expect(udpOverTcpOption).toHaveAttribute('aria-selected', 'true');

      await routes.wireguardSettings.gotoRoot();

      await expectConnected(page);

      await routes.main.expandConnectionPanel();

      const inValue = await routes.main.getInValueText();
      expect(inValue).toMatch(new RegExp(`${escapeRegExp(IN_IP!)}:(80|5001) TCP`));
    });

    for (const port of [80, 5001]) {
      test(`App should show port ${port}`, async () => {
        await gotoUdpOverTcpSettings();
        await routes.udpOverTcpSettings.selectPort(port);
        await routes.udpOverTcpSettings.gotoRoot();
        await routes.main.expandConnectionPanel();

        const inValue = await routes.main.getInValueText();
        expect(inValue).toMatch(`${IN_IP}:${port} TCP`);
      });
    }

    test('App should set obfuscation to automatic', async () => {
      await gotoWireguardSettings();
      await routes.wireguardSettings.selectAutomaticObfuscation();

      const automaticOption = routes.wireguardSettings.getAutomaticObfuscationOption();
      await expect(automaticOption).toHaveAttribute('aria-selected', 'true');
    });
  });

  test('App should connect with Shadowsocks', async () => {
    await exec('mullvad obfuscation set mode shadowsocks');
    await expectConnected(page);
    await exec('mullvad obfuscation set mode off');
    await expectConnected(page);
  });

  test('App should enter blocked state', async () => {
    await exec('mullvad debug block-connection');
    await expectError(page);

    await exec(`mullvad relay set location ${process.env.HOSTNAME}`);
    await expectConnected(page);
  });

  test('App should show multihop', async () => {
    await exec('mullvad relay set tunnel wireguard --use-multihop=on');
    await expectConnected(page);
    const relay = page.getByTestId('hostname-line');
    await expect(relay).toHaveText(
      new RegExp('^' + escapeRegExp(`${process.env.HOSTNAME} via`), 'i'),
    );
    await exec('mullvad relay set tunnel wireguard --use-multihop=off');
    await page.getByText('Disconnect').click();
  });

  test('App should disconnect', async () => {
    await page.getByText('Disconnect').click();
    await expectDisconnected(page);
  });

  test('App should become connected when other frontend connects', async () => {
    await expectDisconnected(page);
    await exec('mullvad connect');
    await expectConnected(page);

    await exec('mullvad disconnect');
    await expectDisconnected(page);
  });
});
