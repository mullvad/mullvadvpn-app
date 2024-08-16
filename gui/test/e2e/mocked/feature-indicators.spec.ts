import { expect, test } from '@playwright/test';
import { Page } from 'playwright';

import { MockedTestUtils, startMockedApp } from './mocked-utils';
import { FeatureIndicator, ILocation, ITunnelEndpoint, TunnelState } from '../../../src/shared/daemon-rpc-types';
import { expectConnected } from '../shared/tunnel-state';

const endpoint: ITunnelEndpoint = {
  address: 'wg10:80',
  protocol: 'tcp',
  quantumResistant: false,
  tunnelType: 'wireguard',
  daita: false,
};

const mockDisconnectedLocation: ILocation = {
  country: 'Sweden',
  city: 'Gothenburg',
  latitude: 58,
  longitude: 12,
  mullvadExitIp: false,
};

const mockConnectedLocation: ILocation = { ...mockDisconnectedLocation, mullvadExitIp: true };

let page: Page;
let util: MockedTestUtils;

test.beforeAll(async () => {
  ({ page, util } = await startMockedApp());
});

test.afterAll(async () => {
  await page.close();
});

test('App should show no feature indicators', async () => {
  await util.mockIpcHandle<ILocation>({
    channel: 'location-get',
    response: mockDisconnectedLocation,
  });
  await util.sendMockIpcResponse<TunnelState>({
    channel: 'tunnel-',
    response: {
      state: 'connected',
      details: { endpoint, location: mockConnectedLocation },
      featureIndicators: undefined,
    },
  });

  await expectConnected(page);
  await expectFeatureIndicators(page, []);

  const ellipsis = page.getByText(/^\d more.../);
  await expect(ellipsis).not.toBeVisible();

  await page.getByTestId('connection-panel-chevron').click();
  await expect(ellipsis).not.toBeVisible();

  await expectFeatureIndicators(page, []);
});

test('App should show feature indicators', async () => {
  await util.mockIpcHandle<ILocation>({
    channel: 'location-get',
    response: mockDisconnectedLocation,
  });
  await util.sendMockIpcResponse<TunnelState>({
    channel: 'tunnel-',
    response: {
      state: 'connected',
      details: { endpoint, location: mockConnectedLocation },
      featureIndicators: [
        FeatureIndicator.daita,
        FeatureIndicator.udp2tcp,
        FeatureIndicator.customMssFix,
        FeatureIndicator.customMtu,
        FeatureIndicator.lanSharing,
        FeatureIndicator.serverIpOverride,
        FeatureIndicator.customDns,
        FeatureIndicator.lockdownMode,
        FeatureIndicator.quantumResistance,
        FeatureIndicator.multihop,
      ],
    },
  });

  await expectConnected(page);
  await expectFeatureIndicators(page, ["DAITA", "Quantum resistance"], false);
  await expectHiddenFeatureIndicator(page, "Mssfix");

  const ellipsis = page.getByText(/^\d more.../);
  await expect(ellipsis).toBeVisible();

  await page.getByTestId('connection-panel-chevron').click();
  await expect(ellipsis).not.toBeVisible();

  await expectFeatureIndicators(page, [
    "DAITA",
    "Quantum resistance",
    "Mssfix",
    "MTU",
    "Obfuscation",
    "Local network sharing",
    "Lockdown mode",
    "Multihop",
    "Custom DNS",
    "Server IP override",
  ]);
});

async function expectHiddenFeatureIndicator(page: Page, hiddenIndicator: string) {
  const indicators = page.getByTestId('feature-indicator');
  const indicator = indicators.getByText(hiddenIndicator, { exact: true });

  await expect(indicator).toHaveCount(1);
  await expect(indicator).not.toBeVisible();
}

async function expectFeatureIndicators(
  page: Page,
  expectedIndicators: Array<string>,
  only = true,
) {
  const indicators = page.getByTestId('feature-indicator');
  if (only) {
    await expect(indicators).toHaveCount(expectedIndicators.length);
  }

  for (const indicator of expectedIndicators) {
    await expect(indicators.getByText(indicator, { exact: true })).toBeVisible();
  }
}
