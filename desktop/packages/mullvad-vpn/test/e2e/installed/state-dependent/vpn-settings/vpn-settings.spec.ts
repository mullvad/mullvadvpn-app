import { execSync } from 'node:child_process';

import { expect, test } from '@playwright/test';
import os from 'os';
import path from 'path';
import { Page } from 'playwright';

import { RoutePath } from '../../../../../src/renderer/lib/routes';
import { fileExists, TestUtils } from '../../../utils';
import { startInstalledApp } from '../../installed-utils';
import { createSelectors } from './helpers';

let page: Page;
let util: TestUtils;
let selectors: ReturnType<typeof createSelectors>;

test.describe('VPN settings', () => {
  const startup = async () => {
    ({ page, util } = await startInstalledApp());
    selectors = createSelectors(page);

    await util.waitForRoute(RoutePath.main);

    await page.click('button[aria-label="Settings"]');
    await util.waitForRoute(RoutePath.settings);
    await page.getByRole('button', { name: 'VPN settings' }).click();
    await util.waitForRoute(RoutePath.vpnSettings);
  };

  test.beforeAll(async () => {
    await startup();
  });

  test.afterAll(async () => {
    await page.close();
  });

  test.describe('Launch on startup and auto-connect', () => {
    test.afterEach(async () => {
      const autoConnectSwitch = selectors.autoConnectSwitch();
      if ((await autoConnectSwitch.getAttribute('aria-checked')) === 'true') {
        await autoConnectSwitch.click();
      }
      await expect(autoConnectSwitch).toHaveAttribute('aria-checked', 'false');

      if (process.platform === 'linux') {
        expect(autoStartPathExists()).toBeFalsy();
      }

      const launchAppOnStartupSwitch = selectors.launchAppOnStartupSwitch();
      if ((await launchAppOnStartupSwitch.getAttribute('aria-checked')) === 'true') {
        await launchAppOnStartupSwitch.click();
      }
      await expect(launchAppOnStartupSwitch).toHaveAttribute('aria-checked', 'false');
    });

    const enableAutoConnect = async () => {
      const autoConnectSwitch = selectors.autoConnectSwitch();
      await expect(autoConnectSwitch).toHaveAttribute('aria-checked', 'false');

      await autoConnectSwitch.click();
      await expect(autoConnectSwitch).toHaveAttribute('aria-checked', 'true');
    };

    const enableLaunchAppOnStartup = async () => {
      const launchAppOnStartupSwitch = selectors.launchAppOnStartupSwitch();
      await expect(launchAppOnStartupSwitch).toHaveAttribute('aria-checked', 'false');

      await launchAppOnStartupSwitch.click();
      await expect(launchAppOnStartupSwitch).toHaveAttribute('aria-checked', 'true');

      if (process.platform === 'linux') {
        expect(autoStartPathExists()).toBeTruthy();
      }
    };

    const getAutoStartPath = () => {
      return path.join(os.homedir(), '.config', 'autostart', 'mullvad-vpn.desktop');
    };

    const autoStartPathExists = () => {
      return fileExists(getAutoStartPath());
    };

    test.describe('Launch app on start-up', () => {
      test('Should be enabled when switch is clicked', async () => {
        await enableAutoConnect();

        const cliAutoConnect = execSync('mullvad auto-connect get').toString();
        expect(cliAutoConnect).toContain('off');
      });

      test('Should not enavble cli auto-connect when enabled alone', () => {
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
      const lanSwitch = selectors.lanSwitch();
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
      const lanSwitch = selectors.lanSwitch();
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

      const lanSwitch = selectors.lanSwitch();
      await lanSwitch.click();

      await expectLocalNetworkSharingEnabled();
    });
  });
});
