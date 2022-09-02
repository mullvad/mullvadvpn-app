import { expect, test } from '@playwright/test';
import { Page } from 'playwright';
import { ElectronApplication } from 'playwright-core';

import { ILocation, ITunnelEndpoint, TunnelState } from '../src/shared/daemon-rpc-types';
import { startApp } from './utils';

let appWindow: Page;
let electronApp: ElectronApplication;

test.beforeAll(async () => {
  const startAppResponse = await startApp();
  appWindow = startAppResponse.appWindow;
  electronApp = startAppResponse.electronApp;
});

test.afterAll(async () => {
  await appWindow.close();
});

test('Validate title', async () => {
  const title = await appWindow.title();
  expect(title).toBe('Mullvad VPN');
  await expect(appWindow.locator('header')).toBeVisible();
});

test('Validate status and header', async () => {
  await electronApp.evaluate(({ ipcMain, webContents }) => {
    const mockLocationResponse = {
      city: 'Uddevalla',
      country: 'Norway',
      latitude: 62,
      longitude: 17,
      mullvadExitIp: false,
    };
    ipcMain.removeHandler('location-get');
    ipcMain.handle('location-get', () => {
      return Promise.resolve({
        type: 'success',
        value: mockLocationResponse,
      });
    });
    webContents.getAllWebContents()[0].send('tunnel-', { state: 'disconnected' } as TunnelState);
  });

  await appWindow.screenshot({ path: 'e2e/screenshots/unsecured.png' });
  const statusSpan = appWindow.locator('span[role="status"]');
  const header = appWindow.locator('header');
  await expect(statusSpan).toContainText('UNSECURED CONNECTION');
  let headerColor = await header.evaluate((el) => {
    return window.getComputedStyle(el).getPropertyValue('background-color');
  });
  expect(headerColor).toBe('rgb(227, 64, 57)');

  await appWindow.locator('text=Secure my connection').click();

  await electronApp.evaluate(({ ipcMain, webContents }) => {
    const mockLocation: ILocation = {
      city: 'Uddevalla',
      country: 'Norway',
      latitude: 62,
      longitude: 17,
      mullvadExitIp: true,
    };
    const mockEndpoint: ITunnelEndpoint = {
      address: 'wg10:80',
      protocol: 'tcp',
      quantumResistant: false,
      tunnelType: 'wireguard',
    };

    ipcMain.removeHandler('location-get');
    ipcMain.handle('location-get', () => {
      return Promise.resolve({
        type: 'success',
        value: mockLocation,
      });
    });

    webContents.getAllWebContents()[0].send('tunnel-', {
      state: 'connected',
      details: {
        endpoint: mockEndpoint,
        location: mockLocation,
      },
    } as TunnelState);
  });

  await expect(statusSpan).toContainText('SECURE CONNECTION');
  headerColor = await header.evaluate((el) => {
    return window.getComputedStyle(el).getPropertyValue('background-color');
  });
  expect(headerColor).toBe('rgb(68, 173, 77)');
  await appWindow.screenshot({ path: 'e2e/screenshots/secure.png' });

  await appWindow.locator('text=Disconnect').click();
  await expect(statusSpan).toContainText('UNSECURED CONNECTION');
});
