import { expect, test } from '@playwright/test';
import { exec as execAsync } from 'child_process';
import { Page } from 'playwright';
import { promisify } from 'util';

import { RoutePath } from '../../../../src/shared/routes';
import { RoutesObjectModel } from '../../route-object-models';
import { expectConnected, expectDisconnected, expectError } from '../../shared/tunnel-state';
import { escapeRegExp, TestUtils } from '../../utils';
import { startInstalledApp } from '../installed-utils';

const exec = promisify(execAsync);

// This test expects the daemon to be logged into an account that has time left and to be
// disconnected. Env parameters:
// CONNECTION_CHECK_URL: Url to the connection check

const { CONNECTION_CHECK_URL } = process.env;

interface ConnectedRelay {
  hostname: string;
  inIp: string;
  inPort: number;
  inProtocol: string;
  obfuscationType?: string;
}

// Returns the relay that the daemon is connected to
async function getConnectedRelay(): Promise<ConnectedRelay> {
  const { stdout } = await exec('mullvad status --json');
  const tunnelState = JSON.parse(stdout);
  expect(tunnelState.state).toBe('connected');

  const { endpoint, location } = tunnelState.details;
  // The in address is that of the obfuscation endpoint if obfuscation is used, otherwise that of
  // the entry relay if multihop is used. This mirrors what the connection panel shows.
  const obfuscation = endpoint.obfuscation?.Single;
  const inEndpoint = obfuscation?.endpoint ?? endpoint.entry_endpoint ?? endpoint;
  const [, inIp, inPort] = /^\[?(.*?)\]?:(\d+)$/.exec(inEndpoint.address)!;

  return {
    hostname: location.hostname,
    inIp,
    inPort: Number(inPort),
    inProtocol: inEndpoint.protocol,
    obfuscationType: obfuscation?.obfuscation_type.toLowerCase(),
  };
}

function formatInAddress(relay: ConnectedRelay) {
  // The app shows IPv6 addresses in brackets
  const inIp = relay.inIp.includes(':') ? `[${relay.inIp}]` : relay.inIp;
  return `${inIp}:${relay.inPort} ${relay.inProtocol.toUpperCase()}`;
}

// Disconnects, applies `changeSettings` and connects again.
async function reconnectWith(changeSettings: () => Promise<unknown>) {
  // Disconnect first so `expectConnected` does not resolve for the old tunnel.
  await exec('mullvad disconnect --wait');
  await expectDisconnected(page);
  await changeSettings();
  await exec('mullvad connect --wait');
  await expectConnected(page);
}

// Expects the connection panel to show the in address (IP, port, protocol) of the relay that the
// daemon is connected to.
async function expectInAddress(relay: ConnectedRelay) {
  const inIp = routes.main.getInIp();
  if (!(await inIp.isVisible())) {
    await routes.main.expandConnectionPanel();
  }
  await expect(inIp).toHaveText(formatInAddress(relay));
}

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
    await util?.closePage();
  });

  test('App should show disconnected tunnel state', async () => {
    await expectDisconnected(page);
  });

  test('App should connect', async () => {
    await page.getByText('Connect', { exact: true }).click();
    await expectConnected(page);

    const relay = routes.main.getRelayHostname();
    const inIp = routes.main.getInIp();
    // If IPv6 is enabled, there will be two "Out" IPs, one for IPv4 and one for IPv6
    // Selecting the first resolves to the IPv4 address regardless of the IP setting
    const outIp = routes.main.getOutIps().first();

    const connectedRelay = await getConnectedRelay();

    await expect(relay).toHaveText(connectedRelay.hostname);
    await expect(inIp).not.toBeVisible();
    await relay.click();

    await expect(inIp).toBeVisible();
    await expect(inIp).toHaveText(formatInAddress(connectedRelay));

    await expect(outIp).toBeVisible();

    const ipResponse = await fetch(`${CONNECTION_CHECK_URL!}/ip`);
    const ip = await ipResponse.text();

    await expect(outIp).toHaveText(ip.trim());
  });

  test('App should show correct WireGuard port', async () => {
    await reconnectWith(async () => {
      await exec('mullvad anti-censorship set mode wireguard-port');
      await exec('mullvad anti-censorship set wireguard-port --port 53');
    });
    const connectedRelay = await getConnectedRelay();
    expect(connectedRelay.inPort).toBe(53);
    await expectInAddress(connectedRelay);

    await reconnectWith(() => exec('mullvad anti-censorship set wireguard-port --port 51820'));
    const newRelay = await getConnectedRelay();
    expect(newRelay.inPort).toBe(51820);
    await expectInAddress(newRelay);

    await reconnectWith(async () => {
      await exec('mullvad anti-censorship set wireguard-port --port any');
      await exec('mullvad anti-censorship set mode auto');
    });
  });

  test.describe('Wireguard UDP-over-TCP', () => {
    async function gotoWireguardSettings() {
      await routes.main.gotoSettings();
      await routes.settings.gotoVpnSettings();
      await routes.vpnSettings.gotoAntiCensorship();
    }

    async function gotoUdpOverTcpSettings() {
      await gotoWireguardSettings();
      await routes.antiCensorship.gotoUdpOverTcpSettings();
    }

    test.beforeAll(async () => {
      await exec('mullvad connect --wait');
    });

    test('App should show UDP', async () => {
      const connectedRelay = await getConnectedRelay();
      await expectInAddress(connectedRelay);
      expect(connectedRelay.inProtocol).toBe('udp');
    });

    test('App should enable UDP-over-TCP', async () => {
      await reconnectWith(async () => {
        await gotoWireguardSettings();

        const udpOverTcpOption = routes.antiCensorship.getUdpOverTcpOption();
        await expect(udpOverTcpOption).toHaveAttribute('aria-selected', 'false');

        await routes.antiCensorship.selectUdpOverTcp();
        await expect(udpOverTcpOption).toHaveAttribute('aria-selected', 'true');

        await routes.antiCensorship.goBackToRoute(RoutePath.main);
      });

      const relay = await getConnectedRelay();
      await expectInAddress(relay);
      expect(relay.obfuscationType).toBe('udp2tcp');
      expect(relay.inProtocol).toBe('tcp');
      expect([80, 443, 5001]).toContain(relay.inPort);
    });

    for (const port of [80, 443, 5001]) {
      test(`App should show port ${port}`, async () => {
        await reconnectWith(async () => {
          await gotoUdpOverTcpSettings();
          await routes.udpOverTcpSettings.selectPort(port);

          await routes.udpOverTcpSettings.goBackToRoute(RoutePath.main);
        });

        const relay = await getConnectedRelay();
        await expectInAddress(relay);
        expect(relay.obfuscationType).toBe('udp2tcp');
        expect(relay.inPort).toBe(port);
      });
    }

    test('App should set obfuscation to automatic', async () => {
      await gotoWireguardSettings();
      await routes.antiCensorship.selectAutomaticObfuscation();

      const automaticOption = routes.antiCensorship.getAutomaticObfuscationOption();
      await expect(automaticOption).toHaveAttribute('aria-selected', 'true');
      await routes.udpOverTcpSettings.goBackToRoute(RoutePath.main);
    });
  });

  test('App should connect with Shadowsocks', async () => {
    await reconnectWith(() => exec('mullvad anti-censorship set mode shadowsocks'));
    const relay = await getConnectedRelay();
    expect(relay.obfuscationType).toBe('shadowsocks');
    await expectInAddress(relay);

    await reconnectWith(() => exec('mullvad anti-censorship set mode off'));
    const relayWithoutObfuscation = await getConnectedRelay();
    expect(relayWithoutObfuscation.obfuscationType).toBeUndefined();
    await expectInAddress(relayWithoutObfuscation);
  });

  test('App should show multihop', async () => {
    await reconnectWith(() => exec('mullvad relay set multihop always'));
    const { hostname } = await getConnectedRelay();
    const relay = routes.main.getRelayHostname();
    await expect(relay).toHaveText(new RegExp('^' + escapeRegExp(`${hostname} via`), 'i'));
    await reconnectWith(() => exec('mullvad relay set multihop auto'));
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

  // This must run last, since `block-connection` overwrites the location constraint
  test('App should enter blocked state', async () => {
    await exec('mullvad debug block-connection');
    await exec('mullvad connect');
    await expectError(page);

    await exec('mullvad disconnect');
    await expectDisconnected(page);
  });
});
