import { expect, test } from '@playwright/test';
import { Page } from 'playwright';

import { getDefaultSettings } from '../../../src/main/default-settings';
import { colorTokens } from '../../../src/renderer/lib/foundations';
import {
  Constraint,
  ErrorStateCause,
  IAccountData,
  IRelayListWithEndpointData,
  ISettings,
  TunnelState,
} from '../../../src/shared/daemon-rpc-types';
import { RoutePath } from '../../../src/shared/routes';
import { getBackgroundColor } from '../utils';
import { MockedTestUtils, startMockedApp } from './mocked-utils';

let page: Page;
let util: MockedTestUtils;

test.beforeAll(async () => {
  ({ page, util } = await startMockedApp());
  await util.waitForRoute(RoutePath.main);
});

test.afterAll(async () => {
  await page.close();
});

/**
 * Expires soon
 */
test('App should notify user about account expiring soon', async () => {
  await util.sendMockIpcResponse<IAccountData>({
    channel: 'account-',
    response: { expiry: new Date(Date.now() + 2 * 24 * 60 * 60 * 1000).toISOString() },
  });

  const title = page.getByTestId('notificationTitle');
  await expect(title).toContainText(/account credit expires soon/i);

  let subTitle = page.getByTestId('notificationSubTitle');
  await expect(subTitle).toContainText(/1 day left\. buy more credit\./i);

  const indicator = page.getByTestId('notificationIndicator');
  const indicatorColor = await getBackgroundColor(indicator);
  expect(indicatorColor).toBe(colorTokens.yellow);

  await util.sendMockIpcResponse<IAccountData>({
    channel: 'account-',
    response: { expiry: new Date(Date.now() + 3 * 24 * 60 * 60 * 1000).toISOString() },
  });
  subTitle = page.getByTestId('notificationSubTitle');
  await expect(subTitle).toContainText(/2 days left\. buy more credit\./i);

  await util.sendMockIpcResponse<IAccountData>({
    channel: 'account-',
    response: { expiry: new Date(Date.now() + 1 * 24 * 60 * 60 * 1000).toISOString() },
  });
  subTitle = page.getByTestId('notificationSubTitle');
  await expect(subTitle).toContainText(/less than a day left\. buy more credit\./i);
});

test.describe('Unsupported wireguard port', () => {
  const portRanges: [number, number][] = [
    [1, 50],
    [51, 100],
  ];
  const portInRange = portRanges[0][0];
  const portOutOfRange = portRanges[1][1] + 1;

  const updatePort = async (port: Constraint<number>) => {
    const settings = getDefaultSettings();
    if ('normal' in settings.relaySettings) {
      settings.relaySettings.normal.wireguardConstraints.port = port;
    }
    await util.sendMockIpcResponse<ISettings>({
      channel: 'settings-',
      response: settings,
    });
  };

  const updatePortRanges = async (portRanges: [number, number][]) => {
    await util.sendMockIpcResponse<IRelayListWithEndpointData>({
      channel: 'relays-',
      response: {
        relayList: {
          countries: [],
        },
        wireguardEndpointData: {
          portRanges,
          udp2tcpPorts: [],
        },
      },
    });
  };

  const updateTunnelState = async (tunnelState: TunnelState) => {
    await util.sendMockIpcResponse<TunnelState>({
      channel: 'tunnel-',
      response: tunnelState,
    });
  };

  test.beforeAll(async () => {
    await updatePortRanges(portRanges);
  });

  const cases: {
    name: string;
    port: Constraint<number>;
    tunnelState: TunnelState;
  }[] = [
    {
      name: 'Should not show notification when any port is allowed',
      port: 'any',
      tunnelState: {
        state: 'error',
        details: { cause: ErrorStateCause.startTunnelError },
      },
    },
    {
      name: 'Should not show notification when port is in range',
      port: { only: portInRange },
      tunnelState: {
        state: 'error',
        details: { cause: ErrorStateCause.startTunnelError },
      },
    },
    {
      name: 'Should not show notification when tunnel is not in error state',
      port: { only: portOutOfRange },
      tunnelState: {
        state: 'connected',
        details: {
          endpoint: {
            address: '',
            daita: false,
            protocol: 'tcp',
            quantumResistant: false,
            tunnelType: 'wireguard',
          },
        },
      },
    },
  ];

  cases.forEach(({ name, port, tunnelState }) => {
    test(name, async () => {
      await updatePort(port);
      await updateTunnelState(tunnelState);

      const subTitle = page.getByTestId('notificationSubTitle');

      await expect(subTitle).not.toContainText(/The selected WireGuard port is not supported/i);
    });
  });

  test('Should show notification when port is out of range', async () => {
    await updatePort({ only: portOutOfRange });
    await updateTunnelState({
      state: 'error',
      details: { cause: ErrorStateCause.startTunnelError },
    });

    const title = page.getByTestId('notificationTitle');
    const subTitle = page.getByTestId('notificationSubTitle');
    await expect(title).toHaveText('BLOCKING INTERNET');
    await expect(subTitle).toContainText(/The selected WireGuard port is not supported/i);
  });
});
