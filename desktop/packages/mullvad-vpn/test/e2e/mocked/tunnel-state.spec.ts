import { expect, test } from '@playwright/test';
import { Page } from 'playwright';

import { ErrorStateCause, ILocation, ITunnelEndpoint } from '../../../src/shared/daemon-rpc-types';
import { RoutesObjectModel } from '../route-object-models';
import {
  expectConnected,
  expectConnecting,
  expectDisconnected,
  expectDisconnecting,
  expectError,
} from '../shared/tunnel-state';
import { MockedTestUtils, startMockedApp } from './mocked-utils';

const mockLocation: ILocation = {
  country: 'Sweden',
  city: 'Gothenburg',
  latitude: 58,
  longitude: 12,
  mullvadExitIp: false,
  ipv4: '127.0.0.1',
  ipv6: '00:00:00:00:00:00:00:01',
};

let page: Page;
let util: MockedTestUtils;
let routes: RoutesObjectModel;

test.describe('Connection states', () => {
  test.beforeAll(async () => {
    ({ page, util } = await startMockedApp());
    routes = new RoutesObjectModel(page, util);
    await routes.main.waitForRoute();
  });

  test.afterAll(async () => {
    await util?.closePage();
  });

  /**
   * Disconnected state
   */
  test('App should show disconnected tunnel state', async () => {
    await util.ipc.tunnel[''].notify({ state: 'disconnected', lockedDown: false });
    await expectDisconnected(page);
  });

  /**
   * Connecting state
   */
  test('App should show connecting tunnel state', async () => {
    await util.ipc.tunnel[''].notify({ state: 'connecting', featureIndicators: undefined });
    await expectConnecting(page);
  });

  /**
   * Disconnecting state
   */
  test('App should show disconnecting tunnel state', async () => {
    await util.ipc.tunnel[''].notify({ state: 'disconnecting', details: 'nothing' });
    await expectDisconnecting(page);
  });

  /**
   * Error state
   */
  test('App should show error tunnel state', async () => {
    await util.ipc.tunnel[''].notify({
      state: 'error',
      details: { cause: ErrorStateCause.isOffline },
    });
    await expectError(page);
  });

  /**
   * Connected state
   */
  test.describe('Connected state', () => {
    test.beforeEach(async () => {
      const location: ILocation = { ...mockLocation, mullvadExitIp: true };

      const endpoint: ITunnelEndpoint = {
        address: 'wg10:80',
        protocol: 'tcp',
        quantumResistant: false,
        daita: false,
      };
      await util.ipc.tunnel[''].notify({
        state: 'connected',
        details: { endpoint, location },
        featureIndicators: undefined,
      });
    });

    test('App should show connected tunnel state', async () => {
      await expectConnected(page);
    });

    test('App should show both IPv4 and IPv6 out address', async () => {
      await routes.main.expandConnectionPanel();

      const outIps = routes.main.getOutIps();
      await expect(outIps).toHaveCount(2);
      await expect(outIps.first()).toHaveText(mockLocation.ipv4!);
      await expect(outIps.last()).toHaveText(mockLocation.ipv6!);
    });
  });
});
