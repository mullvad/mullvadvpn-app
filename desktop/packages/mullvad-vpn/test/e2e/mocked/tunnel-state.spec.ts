import { test } from '@playwright/test';
import { Page } from 'playwright';

import { RoutePath } from '../../../src/renderer/lib/routes';
import {
  ErrorStateCause,
  ILocation,
  ITunnelEndpoint,
  TunnelState,
} from '../../../src/shared/daemon-rpc-types';
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
};

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
 * Disconnected state
 */
test('App should show disconnected tunnel state', async () => {
  await util.mockIpcHandle<ILocation>({
    channel: 'location-get',
    response: mockLocation,
  });
  await util.sendMockIpcResponse<TunnelState>({
    channel: 'tunnel-',
    response: { state: 'disconnected' },
  });
  await expectDisconnected(page);
});

/**
 * Connecting state
 */
test('App should show connecting tunnel state', async () => {
  await util.mockIpcHandle<ILocation>({
    channel: 'location-get',
    response: mockLocation,
  });
  await util.sendMockIpcResponse<TunnelState>({
    channel: 'tunnel-',
    response: { state: 'connecting', featureIndicators: undefined },
  });
  await expectConnecting(page);
});

/**
 * Connected state
 */
test('App should show connected tunnel state', async () => {
  const location: ILocation = { ...mockLocation, mullvadExitIp: true };
  await util.mockIpcHandle<ILocation>({
    channel: 'location-get',
    response: location,
  });

  const endpoint: ITunnelEndpoint = {
    address: 'wg10:80',
    protocol: 'tcp',
    quantumResistant: false,
    tunnelType: 'wireguard',
    daita: false,
  };
  await util.sendMockIpcResponse<TunnelState>({
    channel: 'tunnel-',
    response: { state: 'connected', details: { endpoint, location }, featureIndicators: undefined },
  });

  await expectConnected(page);
});

/**
 * Disconnecting state
 */
test('App should show disconnecting tunnel state', async () => {
  await util.mockIpcHandle<ILocation>({
    channel: 'location-get',
    response: mockLocation,
  });
  await util.sendMockIpcResponse<TunnelState>({
    channel: 'tunnel-',
    response: { state: 'disconnecting', details: 'nothing' },
  });
  await expectDisconnecting(page);
});

/**
 * Error state
 */
test('App should show error tunnel state', async () => {
  await util.mockIpcHandle<ILocation>({
    channel: 'location-get',
    response: mockLocation,
  });
  await util.sendMockIpcResponse<TunnelState>({
    channel: 'tunnel-',
    response: { state: 'error', details: { cause: ErrorStateCause.isOffline } },
  });
  await expectError(page);
});
