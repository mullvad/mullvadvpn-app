import { expect, test } from '@playwright/test';
import { Page } from 'playwright';

import { getDefaultSettings } from '../../../../src/main/default-settings';
import { colorTokens } from '../../../../src/renderer/lib/foundations';
import {
  IRelayListWithEndpointData,
  ISettings,
  IWireguardEndpointData,
  Ownership,
} from '../../../../src/shared/daemon-rpc-types';
import { RoutePath } from '../../../../src/shared/routes';
import { RoutesObjectModel } from '../../route-object-models';
import { MockedTestUtils, startMockedApp } from '../mocked-utils';
import { createHelpers, SelectLocationHelpers } from './helpers';
import { mockData } from './mock-data';

const wireguardEndpointData: IWireguardEndpointData = {
  portRanges: [],
  udp2tcpPorts: [],
};

let page: Page;
let util: MockedTestUtils;
let routes: RoutesObjectModel;
let helpers: SelectLocationHelpers;
const { relayList } = mockData;

test.describe('Select location', () => {
  test.beforeAll(async () => {
    ({ page, util } = await startMockedApp());
    routes = new RoutesObjectModel(page, util);
    helpers = createHelpers(page, routes, util);
    await util.waitForRoute(RoutePath.main);
    await page.getByLabel('Select location').click();
    await util.waitForRoute(RoutePath.selectLocation);

    await util.sendMockIpcResponse<IRelayListWithEndpointData>({
      channel: 'relays-',
      response: { relayList, wireguardEndpointData },
    });
  });

  test.afterAll(async () => {
    await page.close();
  });

  test.describe('Multihop enabled', () => {
    test.beforeAll(async () => {
      const settings = getDefaultSettings();
      if ('normal' in settings.relaySettings) {
        settings.relaySettings.normal.wireguardConstraints.useMultihop = true;
      }

      await util.sendMockIpcResponse<ISettings>({
        channel: 'settings-',
        response: settings,
      });
    });

    test('App should show entry selection', async () => {
      const entryTab = page.getByText('Entry');
      await entryTab.click();
      await expect(entryTab).toHaveCSS('background-color', colorTokens.green);

      const sweden = page.getByText('Sweden');
      await expect(sweden).toBeVisible();
    });

    test('App should show exit selection', async () => {
      const exitTab = page.getByText('Exit');
      await exitTab.click();
      await expect(exitTab).toHaveCSS('background-color', colorTokens.green);

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
      await expect(entryTab).toHaveCSS('background-color', colorTokens.green);

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
      await expect(entryTab).toHaveCSS('background-color', colorTokens.green);

      const sweden = page.getByText('Sweden');
      await expect(sweden).toBeVisible();
    });
  });

  test.describe('Filter', () => {
    test.beforeEach(async () => {
      await helpers.resetView();
      await helpers.resetProviders();
      await helpers.resetOwnership();
    });

    test.describe('Filter by provider', () => {
      test('Should deselect all providers when clicking all providers checkbox', async () => {
        await routes.filter.expandProviders();
        await routes.filter.checkAllProvidersCheckbox();
        expect(await helpers.areAllCheckboxesChecked()).toBe(false);

        await routes.filter.checkAllProvidersCheckbox();
        expect(await helpers.areAllCheckboxesChecked()).toBe(true);
      });

      test('Should apply filter when selecting provider', async () => {
        await routes.filter.expandProviders();
        await routes.filter.checkAllProvidersCheckbox();
        expect(await helpers.areAllCheckboxesChecked()).toBe(false);

        // Select one provider
        const provider = relayList.countries[0].cities[0].relays[0].provider;
        await routes.filter.checkProviderCheckbox(provider);

        await helpers.updateMockRelayFilter({
          providers: [provider],
        });

        await routes.filter.applyFilter();
        await util.waitForRoute(RoutePath.selectLocation);
        const providerFilterChip = routes.selectLocation.getFilterChip('Providers: 1');
        await expect(providerFilterChip).toBeVisible();

        const locatedRelays = helpers.locateRelaysByProvider(relayList, provider);
        const relays = locatedRelays.map((locatedRelay) => locatedRelay.relay);
        const relayNames = relays.map((relay) => relay.hostname);

        // Expand all accordions
        await helpers.expandLocatedRelays(locatedRelays);

        const buttons = routes.selectLocation.getRelaysMatching(relayNames);

        // Expect all filtered relays to have a button
        await expect(buttons).toHaveCount(relays.length);

        // Clear filter
        await providerFilterChip.click();

        // Get all relays and expand accordions
        const allLocatedRelays = helpers.locateRelaysByProvider(relayList);
        await helpers.expandLocatedRelays(allLocatedRelays);

        // Should not have same length as all relays
        await expect(buttons).not.toHaveCount(allLocatedRelays.length);
      });
    });

    test.describe('Filter by ownership', () => {
      test('Should apply filter when selecting ownership', async () => {
        // Select rented only
        await routes.filter.expandOwnership();
        await routes.filter.selectOwnershipOption('Rented only');
        await helpers.updateMockRelayFilter({
          ownership: Ownership.rented,
        });

        await routes.filter.applyFilter();
        await util.waitForRoute(RoutePath.selectLocation);

        const ownerFilterChip = routes.selectLocation.getFilterChip('Rented');
        await expect(ownerFilterChip).toBeVisible();

        const locatedRelays = helpers.locateRelaysByOwner(relayList, false);
        const relays = locatedRelays.map((locatedRelay) => locatedRelay.relay);
        const relayNames = relays.map((relay) => relay.hostname);

        // Expand all accordions
        await helpers.expandLocatedRelays(locatedRelays);

        const buttons = routes.selectLocation.getRelaysMatching(relayNames);

        // Expect all filtered relays to have a button
        await expect(buttons).toHaveCount(relays.length);

        // Clear filter
        await ownerFilterChip.click();

        // Get all relays and expand accordions
        const allLocatedRelays = helpers.locateRelaysByOwner(relayList);
        await helpers.expandLocatedRelays(allLocatedRelays);

        // Should not have same length as all relays
        await expect(buttons).not.toHaveCount(allLocatedRelays.length);
      });
    });
  });
});
