import { expect, test } from '@playwright/test';
import { Page } from 'playwright';

import { getDefaultSettings } from '../../../src/main/default-settings';
import { colorTokens } from '../../../src/renderer/lib/foundations';
import { Constraint, ErrorStateCause, TunnelState } from '../../../src/shared/daemon-rpc-types';
import { RoutesObjectModel } from '../route-object-models';
import { getBackgroundColor } from '../utils';
import { MockedTestUtils, startMockedApp } from './mocked-utils';

let page: Page;
let util: MockedTestUtils;
let routes: RoutesObjectModel;

const startup = async () => {
  ({ page, util } = await startMockedApp());
  routes = new RoutesObjectModel(page, util);

  await routes.main.waitForRoute();
};

/**
 * Expires soon
 */
test.describe('Expiration notifications', () => {
  test.beforeAll(async () => {
    await startup();
  });

  test.afterAll(async () => {
    await util?.closePage();
  });

  test('App should notify user about account expiring soon', async () => {
    await util.ipc.account[''].notify({
      expiry: new Date(Date.now() + 2 * 24 * 60 * 60 * 1000).toISOString(),
    });

    const title = page.getByTestId('notificationTitle');
    await expect(title).toContainText(/account credit expires soon/i);

    let subTitle = page.getByTestId('notificationSubTitle');
    await expect(subTitle).toContainText(/1 day left\. buy more credit\./i);

    const indicator = page.getByTestId('notificationIndicator');
    const indicatorColor = await getBackgroundColor(indicator);
    expect(indicatorColor).toBe(colorTokens.yellow);

    await util.ipc.account[''].notify({
      expiry: new Date(Date.now() + 3 * 24 * 60 * 60 * 1000).toISOString(),
    });
    subTitle = page.getByTestId('notificationSubTitle');
    await expect(subTitle).toContainText(/2 days left\. buy more credit\./i);

    await util.ipc.account[''].notify({
      expiry: new Date(Date.now() + 1 * 24 * 60 * 60 * 1000).toISOString(),
    });
    subTitle = page.getByTestId('notificationSubTitle');
    await expect(subTitle).toContainText(/less than a day left\. buy more credit\./i);
  });
});

test.describe('Unsupported wireguard port', () => {
  test.beforeAll(async () => {
    await startup();
  });

  test.afterAll(async () => {
    await util?.closePage();
  });

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
    await util.ipc.settings[''].notify(settings);
  };

  const updatePortRanges = async (portRanges: [number, number][]) => {
    await util.ipc.relays[''].notify({
      relayList: {
        countries: [],
      },
      wireguardEndpointData: {
        portRanges,
        udp2tcpPorts: [],
      },
    });
  };

  const updateTunnelState = async (tunnelState: TunnelState) => {
    await util.ipc.tunnel[''].notify(tunnelState);
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

      // TODO: Remove these test cases which test for the absence of a specific
      // notification. We prefer to have tests which test for something, rather
      // than the absence of something.
      await expect(async () => {
        const visible = await subTitle.isVisible();
        if (visible) {
          // EITHER: A notification is displayed, but not for unsupported WireGuard port
          expect(await subTitle.innerText()).not.toMatch(
            /The selected WireGuard port is not supported/i,
          );
        } else {
          // OR: no notification is displayed at all
          // NO OP
        }
      }).toPass({
        timeout: 5000,
      });
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

test.describe('App upgrade notifications', () => {
  const sendUpgradeAvailable = async () => {
    await util.ipc.upgradeVersion[''].notify({
      supported: true,
      suggestedIsBeta: false,
      suggestedUpgrade: {
        changelog: [''],
        version: '2025.11',
      },
    });
  };

  const sendUpgradeRollback = async () => {
    await util.ipc.upgradeVersion[''].notify({
      supported: true,
      suggestedIsBeta: false,
      suggestedUpgrade: undefined,
    });
  };

  test.beforeAll(async () => {
    await startup();
  });

  test.afterAll(async () => {
    await util?.closePage();
  });

  test('App upgrade notification displayed on new version', async () => {
    const notificationTitle = routes.main.getNotificationTitle();
    await expect(notificationTitle).not.toBeVisible();

    await sendUpgradeAvailable();
    await expect(notificationTitle).toBeVisible();
    await expect(notificationTitle).toHaveText('UPDATE AVAILABLE');
  });

  test('App upgrade notification handles new version rollback', async () => {
    await sendUpgradeAvailable();
    const notificationTitle = routes.main.getNotificationTitle();
    await expect(notificationTitle).toHaveText('UPDATE AVAILABLE');

    await sendUpgradeRollback();
    await expect(notificationTitle).not.toBeVisible();
  });
});
