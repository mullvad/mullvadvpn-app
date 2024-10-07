import path from 'path';
import { execSync } from 'child_process';
import { expect, Page, test } from '@playwright/test';
import { startInstalledApp } from '../installed-utils';
import { fileExists, TestUtils } from '../../utils';

if (process.env.HOME === undefined) {
  throw new Error('$HOME not set');
}

const AUTOSTART_PATH = path.join(process.env.HOME, '.config', 'autostart', 'mullvad-vpn.desktop');

let page: Page;
let util: TestUtils;

test.beforeAll(async () => {
  ({ page, util } = await startInstalledApp());
});

test.afterAll(async () => {
  await page.close();
});

test.describe('VPN Settings', () => {
  test('Auto-connect setting', async () => {
    // Navigate to the VPN settings view
    await util.waitForNavigation(async () => await page.click('button[aria-label="Settings"]'));
    await util.waitForNavigation(async () => await page.click('text=VPN settings'));

    // Find the auto-connect toggle
    const autoConnectToggle = page.getByText('Auto-connect').locator('..').getByRole('checkbox');

    // Check initial state
    const initialCliState = execSync('mullvad auto-connect get').toString().trim();
    expect(initialCliState).toMatch(/off$/);
    await expect(autoConnectToggle).toHaveAttribute('aria-checked', 'false')

    // Toggle auto-connect
    await autoConnectToggle.click();

    // Verify the setting was applied correctly
    await expect(autoConnectToggle).toHaveAttribute('aria-checked', 'true')
    const newCliState = execSync('mullvad auto-connect get').toString().trim();
    expect(newCliState).toMatch(/off$/);
  });

  test('Launch on startup setting', async () => {
    // Find the launch on start-up toggle
    const launchOnStartupToggle =
      page.getByText('Launch app on start-up').locator('..').getByRole('checkbox');

    // Check initial state
    const initialCliState = execSync('mullvad auto-connect get').toString().trim();
    expect(initialCliState).toMatch(/off$/);
    await expect(launchOnStartupToggle).toHaveAttribute('aria-checked', 'false')
    expect(fileExists(AUTOSTART_PATH)).toBeFalsy();

    // Toggle launch on start-up
    await launchOnStartupToggle.click();

    // Verify the setting was applied correctly
    await expect(launchOnStartupToggle).toHaveAttribute('aria-checked', 'true')
    expect(fileExists(AUTOSTART_PATH)).toBeTruthy();
    const newCliState = execSync('mullvad auto-connect get').toString().trim();
    expect(newCliState).toMatch(/on$/);

    await launchOnStartupToggle.click();

    // Toggle auto-connect back off
    // NOTE: This must be done to clean up the auto-start file
    // TODO: Reset GUI settings between all tests
    const autoConnectToggle = page.getByText('Auto-connect').locator('..').getByRole('checkbox');
    await autoConnectToggle.click();
  });

  test('LAN settings', async () => {
    // Find the LAN toggle
    const lanToggle = page.getByText('Local network sharing').locator('..').getByRole('checkbox');

    // Check initial state
    const initialCliState = execSync('mullvad lan get').toString().trim();
    expect(initialCliState).toMatch(/block$/);
    await expect(lanToggle).toHaveAttribute('aria-checked', 'false')

    // Toggle LAN setting
    await lanToggle.click();

    // Verify the setting was applied correctly
    await expect(lanToggle).toHaveAttribute('aria-checked', 'true')
    const newState = execSync('mullvad lan get').toString().trim();
    expect(newState).toMatch(/allow$/);
  });
});
