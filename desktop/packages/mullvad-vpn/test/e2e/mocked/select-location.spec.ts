import { expect, test } from '@playwright/test';
import { Page } from 'playwright';

import { getDefaultSettings } from '../../../src/main/default-settings';
import { colors } from '../../../src/renderer/lib/foundations';
import { RoutePath } from '../../../src/renderer/lib/routes';
import {
  IRelayList,
  IRelayListWithEndpointData,
  ISettings,
  IWireguardEndpointData,
} from '../../../src/shared/daemon-rpc-types';
import { MockedTestUtils, startMockedApp } from './mocked-utils';

const relayList: IRelayList = {
  countries: [
    {
      name: 'Sweden',
      code: 'se',
      cities: [
        {
          name: 'Gothenburg',
          code: 'got',
          latitude: 58,
          longitude: 12,
          relays: [
            {
              hostname: 'se-got-wg-101',
              provider: 'mullvad',
              ipv4AddrIn: '10.0.0.1',
              includeInCountry: true,
              active: true,
              weight: 0,
              owned: true,
              endpointType: 'wireguard',
              daita: true,
            },
            {
              hostname: 'se-got-wg-102',
              provider: 'mullvad',
              ipv4AddrIn: '10.0.0.2',
              includeInCountry: true,
              active: true,
              weight: 0,
              owned: true,
              endpointType: 'wireguard',
              daita: true,
            },
          ],
        },
      ],
    },
  ],
};

const wireguardEndpointData: IWireguardEndpointData = {
  portRanges: [],
  udp2tcpPorts: [],
};

let page: Page;
let util: MockedTestUtils;

test.beforeAll(async () => {
  ({ page, util } = await startMockedApp());
  await util.waitForRoute(RoutePath.main);
  await setMultihop();
  await page.getByLabel('Select location').click();
  await util.waitForRoute(RoutePath.selectLocation);
});

test.afterAll(async () => {
  await page.close();
});

async function setMultihop() {
  const settings = getDefaultSettings();
  if ('normal' in settings.relaySettings) {
    settings.relaySettings.normal.wireguardConstraints.useMultihop = true;
  }

  await util.sendMockIpcResponse<ISettings>({
    channel: 'settings-',
    response: settings,
  });

  await util.sendMockIpcResponse<IRelayListWithEndpointData>({
    channel: 'relays-',
    response: { relayList, wireguardEndpointData },
  });
}

test('App should show entry selection', async () => {
  const entryTab = page.getByText('Entry');
  await entryTab.click();
  await expect(entryTab).toHaveCSS('background-color', colors['--color-green']);

  const sweden = page.getByText('Sweden');
  await expect(sweden).toBeVisible();
});

test('App should show exit selection', async () => {
  const exitTab = page.getByText('Exit');
  await exitTab.click();
  await expect(exitTab).toHaveCSS('background-color', colors['--color-green']);

  const sweden = page.getByText('Sweden');
  await expect(sweden).toBeVisible();
});

test("App shouldn't show entry selection when daita is enabled without direct only", async () => {
  const settings = getDefaultSettings();
  if ('normal' in settings.relaySettings && settings.tunnelOptions.wireguard.daita) {
    settings.relaySettings.normal.wireguardConstraints.useMultihop = true;
    settings.tunnelOptions.wireguard.daita.enabled = true;
    settings.tunnelOptions.wireguard.daita.directOnly = false;
  }

  await util.sendMockIpcResponse<ISettings>({
    channel: 'settings-',
    response: settings,
  });

  const entryTab = page.getByText('Entry').first();
  await entryTab.click();
  await expect(entryTab).toHaveCSS('background-color', colors['--color-green']);

  const sweden = page.getByText('Sweden');
  await expect(sweden).not.toBeVisible();
});

test('App should show entry selection when daita is enabled with direct only', async () => {
  const settings = getDefaultSettings();
  if ('normal' in settings.relaySettings && settings.tunnelOptions.wireguard.daita) {
    settings.relaySettings.normal.wireguardConstraints.useMultihop = true;
    settings.tunnelOptions.wireguard.daita.enabled = true;
    settings.tunnelOptions.wireguard.daita.directOnly = true;
  }

  await util.sendMockIpcResponse<ISettings>({
    channel: 'settings-',
    response: settings,
  });

  const entryTab = page.getByText('Entry');
  await entryTab.click();
  await expect(entryTab).toHaveCSS('background-color', colors['--color-green']);

  const sweden = page.getByText('Sweden');
  await expect(sweden).toBeVisible();
});
