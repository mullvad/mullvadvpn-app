import { execSync } from 'node:child_process';

import { expect, test } from '@playwright/test';
import { Page } from 'playwright';

import { RoutesObjectModel } from '../../../route-object-models';
import { TestUtils } from '../../../utils';
import { startInstalledApp } from '../../installed-utils';
import { autoStartPathExists } from './helpers';

let page: Page;
let util: TestUtils;
let routes: RoutesObjectModel;

test.describe('VPN settings', () => {
  const startup = async () => {
    ({ page, util } = await startInstalledApp());
    routes = new RoutesObjectModel(page, util);

    await routes.main.waitForRoute();
    await routes.main.gotoSettings();
    await routes.settings.gotoVpnSettings();
  };

  test.beforeAll(async () => {
    await startup();
  });

  test.afterAll(async () => {
    await page.close();
  });

  test.describe('Launch on startup and auto-connect', () => {
    test.beforeAll(() => {
      if (process.platform === 'linux') {
        expect(autoStartPathExists()).toBeFalsy();
      }
    });
    test.afterEach(async () => {
      await routes.vpnSettings.setAutoConnectSwitch(false);
      const autoConnectSwitchChecked = await routes.vpnSettings.getAutoConnectSwitchState();
      expect(autoConnectSwitchChecked).toBe('false');

      await routes.vpnSettings.setLaunchAppOnStartupSwitch(false);
      const launchOnStartupSwitchChecked =
        await routes.vpnSettings.getLaunchAppOnStartupSwitchState();
      expect(launchOnStartupSwitchChecked).toBe('false');
    });

    const enableAutoConnect = async () => {
      await routes.vpnSettings.setAutoConnectSwitch(true);
      const autoConnectSwitchChecked = await routes.vpnSettings.getAutoConnectSwitchState();
      expect(autoConnectSwitchChecked).toBe('true');
    };

    const enableLaunchAppOnStartup = async () => {
      await routes.vpnSettings.setLaunchAppOnStartupSwitch(true);
      const launchOnStartupSwitchChecked =
        await routes.vpnSettings.getLaunchAppOnStartupSwitchState();
      expect(launchOnStartupSwitchChecked).toBe('true');

      if (process.platform === 'linux') {
        expect(autoStartPathExists()).toBeTruthy();
      }
    };

    test.describe('Launch app on start-up', () => {
      test('Should be enabled when switch is clicked', async () => {
        await enableAutoConnect();

        const cliAutoConnect = execSync('mullvad auto-connect get').toString();
        expect(cliAutoConnect).toContain('off');
      });

      test('Should not enable cli auto-connect when enabled alone', () => {
        const cliAutoConnect = execSync('mullvad auto-connect get').toString();
        expect(cliAutoConnect).toContain('off');
      });
    });

    test.describe('Auto-connect', () => {
      test('Should be enabled when switch is clicked', async () => {
        await enableLaunchAppOnStartup();
      });

      test('Should not enable cli auto-connect when enabled alone', () => {
        const cliAutoConnect = execSync('mullvad auto-connect get').toString();
        expect(cliAutoConnect).toContain('off');
      });
    });

    test('Should enable cli auto-connect when both launch app on start-up and auto-connect are enabled', async () => {
      await enableLaunchAppOnStartup();
      await enableAutoConnect();

      const cliAutoConnect = execSync('mullvad auto-connect get').toString();
      expect(cliAutoConnect).toContain('on');
    });
  });

  test.describe('LAN settings', () => {
    const expectLocalNetworkSharing = async (
      ariaChecked: 'true' | 'false',
      cliState: 'allow' | 'block',
    ) => {
      const lanSwitch = routes.vpnSettings.getLanSwitch();
      await expect(lanSwitch).toHaveAttribute('aria-checked', ariaChecked);
      const cliStateOutput = execSync('mullvad lan get').toString();
      expect(cliStateOutput).toContain(cliState);
    };

    const expectLocalNetworkSharingEnabled = async () => {
      await expectLocalNetworkSharing('true', 'allow');
    };

    const expectLocalNetworkSharingDisabled = async () => {
      await expectLocalNetworkSharing('false', 'block');
    };

    const disableLocalNetworkSharing = async () => {
      const lanSwitch = routes.vpnSettings.getLanSwitch();
      if ((await lanSwitch.getAttribute('aria-checked')) === 'true') {
        await lanSwitch.click();
      }
      await expectLocalNetworkSharingDisabled();
    };

    test.beforeAll(async () => {
      // Ensure local network sharing is disabled before starting the tests
      await disableLocalNetworkSharing();
    });

    test.afterEach(async () => {
      await disableLocalNetworkSharing();
    });

    test('Should be enabled when switch is clicked', async () => {
      await expectLocalNetworkSharingDisabled();

      const lanSwitch = routes.vpnSettings.getLanSwitch();
      await lanSwitch.click();

      await expectLocalNetworkSharingEnabled();
    });
  });
});
