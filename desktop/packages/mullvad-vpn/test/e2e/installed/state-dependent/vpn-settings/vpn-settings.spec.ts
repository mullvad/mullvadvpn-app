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
    test.afterEach(async () => {
      const autoConnectSwitch = await routes.vpnSettings.setAutoConnectSwitch(false);
      await expect(autoConnectSwitch).toHaveAttribute('aria-checked', 'false');

      if (process.platform === 'linux') {
        expect(autoStartPathExists()).toBeFalsy();
      }

      const launchAppOnStartupSwitch = await routes.vpnSettings.setLaunchAppOnStartupSwitch(false);
      await expect(launchAppOnStartupSwitch).toHaveAttribute('aria-checked', 'false');
    });

    const enableAutoConnect = async () => {
      const autoConnectSwitch = await routes.vpnSettings.setAutoConnectSwitch(true);
      await expect(autoConnectSwitch).toHaveAttribute('aria-checked', 'true');
    };

    const enableLaunchAppOnStartup = async () => {
      const launchAppOnStartupSwitch = await routes.vpnSettings.setLaunchAppOnStartupSwitch(true);
      await expect(launchAppOnStartupSwitch).toHaveAttribute('aria-checked', 'true');

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
