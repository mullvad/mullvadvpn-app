import { expect, test } from '@playwright/test';
import { execSync } from 'child_process';
import { Page } from 'playwright';

import { RoutePath } from '../../../../src/shared/routes';
import { RoutesObjectModel } from '../../route-object-models';
import { TestUtils } from '../../utils';
import { startInstalledApp } from '../installed-utils';

const SHADOWSOCKS_PORT = 65_000;
const UDPOVERTCP_PORT = '80';

// This test sets different obfuscation settings combinations and verifies that it was set in the
// daemon.

let page: Page;
let util: TestUtils;
let routes: RoutesObjectModel;

test.beforeAll(async () => {
  ({ page, util } = await startInstalledApp());
  routes = new RoutesObjectModel(page, util);
  await util.expectRoute(RoutePath.main);
  await routes.main.gotoSettings();
  await routes.settings.gotoVpnSettings();
  await routes.vpnSettings.gotoCensorshipCircumvention();
});

test.afterAll(async () => {
  await util?.closePage();
});

test('App should have automatic obfuscation', async () => {
  const automaticObfuscationOption =
    routes.wireguardSettings.selectors.automaticObfuscationOption();

  await expect(automaticObfuscationOption).toHaveAttribute('aria-selected', 'true');

  const cliObfuscation = execSync('mullvad obfuscation get').toString().split('\n');
  expect(cliObfuscation[0]).toEqual('Obfuscation mode: auto');
  expect(cliObfuscation[1]).toEqual('udp2tcp settings: any port');
  expect(cliObfuscation[2]).toEqual('Shadowsocks settings: any port');
});

test('App should set obfuscation to shadowsocks with custom port', async () => {
  await routes.wireguardSettings.gotoShadowSocksSettings();
  const automaticOption = routes.shadowsocksSettings.selectors.automaticPortOption();

  await expect(automaticOption).toHaveAttribute('aria-selected', 'true');

  await routes.shadowsocksSettings.fillPortInput(SHADOWSOCKS_PORT);

  const customPortOption = routes.shadowsocksSettings.selectors.customPortOption();
  await expect(customPortOption).toHaveAttribute('aria-selected', 'true');

  await routes.shadowsocksSettings.goBack();

  const shadowsocksOption = routes.wireguardSettings.selectors.shadowsocksOption();
  await expect(shadowsocksOption).toContainText(`Port: ${SHADOWSOCKS_PORT}`);

  await routes.wireguardSettings.selectShadowsocks();
  await expect(shadowsocksOption).toHaveAttribute('aria-selected', 'true');

  const cliObfuscation = execSync('mullvad obfuscation get').toString().split('\n')[2];
  expect(cliObfuscation).toEqual(`Shadowsocks settings: port ${SHADOWSOCKS_PORT}`);
});

test('App should still have shadowsocks custom port', async () => {
  await routes.wireguardSettings.gotoShadowSocksSettings();

  const customPortOption = routes.shadowsocksSettings.selectors.customPortOption();
  await expect(customPortOption).toHaveAttribute('aria-selected', 'true');

  await routes.shadowsocksSettings.goBack();
});

test('App should set obfuscation to UDP-over-TCP with port', async () => {
  await routes.wireguardSettings.gotoUdpOverTcpSettings();

  const automaticOption = routes.udpOverTcpSettings.selectors.automaticPortOption();
  await expect(automaticOption).toHaveAttribute('aria-selected', 'true');

  await routes.udpOverTcpSettings.selectPort(parseInt(UDPOVERTCP_PORT));
  const portButton = routes.udpOverTcpSettings.selectors.portNumber(parseInt(UDPOVERTCP_PORT));
  await expect(portButton).toHaveAttribute('aria-selected', 'true');

  await routes.udpOverTcpSettings.goBack();

  const udpOverTcpItem = routes.wireguardSettings.selectors.udpOverTcpOption();
  await routes.wireguardSettings.selectUdpOverTcp();
  await expect(udpOverTcpItem).toHaveAttribute('aria-selected', 'true');
  await expect(udpOverTcpItem).toContainText(`Port: ${UDPOVERTCP_PORT}`);

  const cliObfuscation = execSync('mullvad obfuscation get').toString().split('\n')[1];
  expect(cliObfuscation).toEqual(`udp2tcp settings: port ${UDPOVERTCP_PORT}`);
});
