import { expect, test } from '@playwright/test';
import { Page } from 'playwright';

import { getDefaultSettings } from '../../../../src/main/default-settings';
import { Constraint } from '../../../../src/shared/daemon-rpc-types';
import { RoutesObjectModel } from '../../route-object-models';
import { MockedTestUtils, startMockedApp } from '../mocked-utils';

let page: Page;
let util: MockedTestUtils;
let routes: RoutesObjectModel;

const VALID_PORT = 12345;
const INVALID_PORT = 123;

test.describe('WireGuard port settings', () => {
  const startup = async () => {
    ({ page, util } = await startMockedApp());
    routes = new RoutesObjectModel(page, util);

    await routes.main.waitForRoute();

    await routes.main.gotoSettings();
    await routes.settings.gotoVpnSettings();
    await routes.vpnSettings.gotoAntiCensorship();
    await routes.antiCensorship.gotoWireguardPort();
  };

  test.beforeAll(async () => {
    await startup();

    await util.ipc.relays[''].notify({
      relayList: {
        countries: [],
      },
      wireguardEndpointData: { portRanges: [[12344, 12346]], udp2tcpPorts: [12344] },
    });
  });

  test.afterAll(async () => {
    await util?.closePage();
  });

  const setPort = async (port: Constraint<number>) => {
    const defaultSettings = getDefaultSettings();

    await util.ipc.settings[''].notify({
      ...defaultSettings,
      obfuscationSettings: {
        ...defaultSettings.obfuscationSettings,
        wireGuardPortSettings: {
          port,
        },
      },
    });
  };

  test.beforeEach(async () => {
    const input = routes.wireguardPort.selectors.customInput();
    await input.fill('');
    await setPort('any');
  });

  test('Should select automatic port', async () => {
    await setPort({ only: 51820 });

    const option = routes.wireguardPort.selectors.automaticOption();
    await Promise.all([util.ipc.settings.setObfuscationSettings.expect(), option.click()]);

    await setPort('any');
    await expect(option).toHaveAttribute('aria-selected', 'true');
  });

  test('Should select port 51820', async () => {
    const option = routes.wireguardPort.selectors.fiveOneEightTwoZeroOption();
    await Promise.all([util.ipc.settings.setObfuscationSettings.expect(), option.click()]);
    await setPort({ only: 51820 });
    await expect(option).toHaveAttribute('aria-selected', 'true');
  });

  test('Should select port 53', async () => {
    const option = routes.wireguardPort.selectors.fiveThreeOption();
    await Promise.all([util.ipc.settings.setObfuscationSettings.expect(), option.click()]);
    await setPort({ only: 53 });
    await expect(option).toHaveAttribute('aria-selected', 'true');
  });

  test('Should selecting custom port option should focus input', async () => {
    const option = routes.wireguardPort.selectors.customOption();
    await option.click();

    const input = routes.wireguardPort.selectors.customInput();
    await expect(input).toBeFocused();
  });

  test('Should set custom port', async () => {
    const option = routes.wireguardPort.selectors.customOption();
    await option.click();

    const input = routes.wireguardPort.selectors.customInput();
    await input.fill(VALID_PORT.toString());
    await Promise.all([util.ipc.settings.setObfuscationSettings.expect(), input.press('Enter')]);

    await setPort({ only: VALID_PORT });
    await expect(option).toHaveAttribute('aria-selected', 'true');
  });

  test('Should not accept invalid custom port', async () => {
    const option = routes.wireguardPort.selectors.customOption();
    await option.click();

    const input = routes.wireguardPort.selectors.customInput();
    await input.fill('9999');
    await input.press('Enter');

    await expect(input).toHaveAttribute('aria-invalid', 'true');
  });

  test('Should select custom option when clicking trigger with valid value', async () => {
    const option = routes.wireguardPort.selectors.customOption();
    await option.click();

    const input = routes.wireguardPort.selectors.customInput();
    await input.fill(VALID_PORT.toString());

    await Promise.all([util.ipc.settings.setObfuscationSettings.expect(), option.click()]);

    await setPort({ only: VALID_PORT });
    await expect(option).toHaveAttribute('aria-selected', 'true');
  });

  test('Should not select custom option when clicking input with valid value', async () => {
    const input = routes.wireguardPort.selectors.customInput();
    await input.fill(VALID_PORT.toString());
    await input.click();

    const option = routes.wireguardPort.selectors.customOption();
    await expect(option).not.toHaveAttribute('aria-selected', 'true');

    const automaticOption = routes.wireguardPort.selectors.automaticOption();
    await expect(automaticOption).toHaveAttribute('aria-selected', 'true');
  });

  test('Should reset custom port on blur', async () => {
    const input = routes.wireguardPort.selectors.customInput();

    await input.fill(VALID_PORT.toString());
    await expect(input).toHaveAttribute('aria-invalid', 'false');
    await input.blur();

    await expect(input).toHaveValue('');

    await input.fill(INVALID_PORT.toString());
    await expect(input).toHaveAttribute('aria-invalid', 'true');
    await input.blur();

    await expect(input).toHaveValue('');
  });
});
