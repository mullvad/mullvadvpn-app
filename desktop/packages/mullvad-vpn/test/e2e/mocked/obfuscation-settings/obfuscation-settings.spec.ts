import { expect, test } from '@playwright/test';
import { Page } from 'playwright';

import { getDefaultSettings } from '../../../../src/main/default-settings';
import { ObfuscationType } from '../../../../src/shared/daemon-rpc-types';
import { RoutesObjectModel } from '../../route-object-models';
import { MockedTestUtils, startMockedApp } from '../mocked-utils';

let page: Page;
let util: MockedTestUtils;
let routes: RoutesObjectModel;

test.describe('Obfuscation settings', () => {
  const startup = async () => {
    ({ page, util } = await startMockedApp());
    routes = new RoutesObjectModel(page, util);

    await routes.main.waitForRoute();

    await routes.main.gotoSettings();
    await routes.settings.gotoVpnSettings();
    await routes.vpnSettings.gotoObfuscationSettings();
  };

  test.beforeAll(async () => {
    await startup();
  });

  test.afterAll(async () => {
    await util?.closePage();
  });

  const setObfuscation = async (type: ObfuscationType) => {
    const defaultSettings = getDefaultSettings();
    await util.ipc.settings[''].notify({
      ...defaultSettings,
      obfuscationSettings: {
        ...defaultSettings.obfuscationSettings,
        selectedObfuscation: type,
      },
    });
  };

  test('Should select automatic obfuscation', async () => {
    await setObfuscation(ObfuscationType.off);

    const option = routes.obfuscationSettings.selectors.automaticObfuscationOption();
    await Promise.all([util.ipc.settings.setObfuscationSettings.expect(), option.click()]);

    await setObfuscation(ObfuscationType.auto);
    await expect(option).toHaveAttribute('aria-selected', 'true');
  });

  test('Should select LWO obfuscation', async () => {
    const option = routes.obfuscationSettings.selectors.lwoOption();
    await Promise.all([util.ipc.settings.setObfuscationSettings.expect(), option.click()]);

    await setObfuscation(ObfuscationType.lwo);
    await expect(option).toHaveAttribute('aria-selected', 'true');
  });

  test('Should select QUIC obfuscation', async () => {
    const option = routes.obfuscationSettings.selectors.quicOption();
    await Promise.all([util.ipc.settings.setObfuscationSettings.expect(), option.click()]);

    await setObfuscation(ObfuscationType.quic);
    await expect(option).toHaveAttribute('aria-selected', 'true');
  });

  test('Should select shadowsocks obfuscation', async () => {
    const option = routes.obfuscationSettings.selectors.shadowsocksOption();
    await Promise.all([util.ipc.settings.setObfuscationSettings.expect(), option.click()]);

    await setObfuscation(ObfuscationType.shadowsocks);
    await expect(option).toHaveAttribute('aria-selected', 'true');
  });

  test('Should select udp-over-tcp obfuscation', async () => {
    const option = routes.obfuscationSettings.selectors.udpOverTcpOption();
    await Promise.all([util.ipc.settings.setObfuscationSettings.expect(), option.click()]);

    await setObfuscation(ObfuscationType.udp2tcp);
    await expect(option).toHaveAttribute('aria-selected', 'true');
  });
});
