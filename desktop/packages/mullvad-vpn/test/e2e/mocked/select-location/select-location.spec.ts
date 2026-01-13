import { expect, test } from '@playwright/test';
import { Page } from 'playwright';

import { getDefaultSettings } from '../../../../src/main/default-settings';
import { colorTokens } from '../../../../src/renderer/lib/foundations';
import { ObfuscationType, Ownership } from '../../../../src/shared/daemon-rpc-types';
import { RoutePath } from '../../../../src/shared/routes';
import { mockData } from '../../mock-data';
import { RoutesObjectModel } from '../../route-object-models';
import { MockedTestUtils, startMockedApp } from '../mocked-utils';
import { createHelpers, SelectLocationHelpers } from './helpers';

const { relayList } = mockData;

let page: Page;
let util: MockedTestUtils;
let routes: RoutesObjectModel;
let helpers: SelectLocationHelpers;

test.describe('Select location', () => {
  test.beforeAll(async () => {
    ({ page, util } = await startMockedApp());
    routes = new RoutesObjectModel(page, util);
    helpers = createHelpers(page, routes, util);

    await util.expectRoute(RoutePath.main);
  });

  test.beforeEach(async () => {
    if ((await util.getCurrentRoute()) === RoutePath.main) {
      await routes.main.gotoSelectLocation();
    }
  });

  test.afterAll(async () => {
    await util?.closePage();
  });

  test('Should focus search input on load', async () => {
    const input = routes.selectLocation.selectors.searchInput();
    await expect(input).toBeFocused();
  });

  test.describe('Multihop enabled', () => {
    test.beforeAll(async () => {
      await helpers.updateMockSettings({
        multihop: true,
      });
    });

    test.beforeEach(async () => {
      await routes.selectLocation.selectors.entryButton().click();
    });

    test('App should show entry selection', async () => {
      const entryButton = routes.selectLocation.selectors.entryButton();
      await expect(entryButton).toHaveCSS('background-color', colorTokens.green);

      const sweden = page.getByText('Sweden');
      await expect(sweden).toBeVisible();
    });

    test('App should show exit selection', async () => {
      const exitButton = routes.selectLocation.selectors.exitButton();
      await exitButton.click();
      await expect(exitButton).toHaveCSS('background-color', colorTokens.green);

      const sweden = page.getByText('Sweden');
      await expect(sweden).toBeVisible();
    });

    test("App shouldn't show entry selection when daita is enabled without direct only", async () => {
      await helpers.updateMockSettings({
        multihop: true,
        daita: true,
        directOnly: false,
      });

      const entryButton = routes.selectLocation.selectors.entryButton();
      await expect(entryButton).toHaveCSS('background-color', colorTokens.green);

      const sweden = page.getByText('Sweden');
      await expect(sweden).not.toBeVisible();
    });

    test('App should show entry selection when daita is enabled with direct only', async () => {
      await helpers.updateMockSettings({
        multihop: true,
        daita: true,
        directOnly: true,
      });

      const entryButton = routes.selectLocation.selectors.entryButton();
      await expect(entryButton).toHaveCSS('background-color', colorTokens.green);

      const sweden = page.getByText('Sweden');
      await expect(sweden).toBeVisible();
    });

    test('Should show only wireguard servers in entry list', async () => {
      const entryButton = routes.selectLocation.selectors.entryButton();
      await entryButton.click();

      const wireguardRelays = relayList.countries[0].cities[0].relays;
      const hostnames = wireguardRelays.map((relay) => relay.hostname);
      const relaySelectionPaths = helpers.toSelectionPaths(
        helpers.getRelaysByHostnames(relayList, hostnames),
      );

      await helpers.expandLocatedRelays(relaySelectionPaths);

      const buttons = routes.selectLocation.selectors.relaysMatching(hostnames);
      await expect(buttons).toHaveCount(wireguardRelays.length);
    });

    test('Should show only wireguard servers in exit list', async () => {
      const exitButton = routes.selectLocation.selectors.exitButton();
      await exitButton.click();

      const wireguardRelays = relayList.countries[0].cities[0].relays;
      const hostnames = wireguardRelays.map((relay) => relay.hostname);
      const relaySelectionPaths = helpers.toSelectionPaths(
        helpers.getRelaysByHostnames(relayList, hostnames),
      );

      await helpers.expandLocatedRelays(relaySelectionPaths);

      const buttons = routes.selectLocation.selectors.relaysMatching(hostnames);
      await expect(buttons).toHaveCount(wireguardRelays.length);
    });

    test('Should disable entry server in exit list', async () => {
      await util.ipc.tunnel.connect.ignore();
      await util.ipc.settings.setRelaySettings.ignore();

      const settings = await helpers.updateMockSettings({
        multihop: true,
        daita: true,
        directOnly: true,
      });

      const entryButton = routes.selectLocation.selectors.entryButton();
      await entryButton.click();

      // Get first wireguard relay
      const [entryRelay, exitRelay] = relayList.countries[0].cities[0].relays;

      if (!entryRelay) {
        throw new Error('No wireguard relay found in mocked data');
      }

      const relaySelectionPaths = helpers.toSelectionPaths(
        helpers.getRelaysByHostnames(relayList, [entryRelay.hostname]),
      );

      await helpers.expandLocatedRelays(relaySelectionPaths);

      await routes.selectLocation.selectors.relaysMatching([entryRelay.hostname]).first().click();

      await helpers.updateEntryLocation(relaySelectionPaths[0], settings);
      await helpers.expandLocatedRelays(relaySelectionPaths);
      const entryRelayButton = routes.selectLocation.selectors.relaysMatching([
        entryRelay.hostname,
      ]);
      await expect(entryRelayButton).toBeDisabled();

      const relaySelectionPathsExit = helpers.toSelectionPaths(
        helpers.getRelaysByHostnames(relayList, [exitRelay.hostname]),
      );
      await helpers.expandLocatedRelays(relaySelectionPathsExit);

      // Clicking exit relay should navigate to main route
      const exitRelayButton = routes.selectLocation.selectors.relaysMatching([exitRelay.hostname]);
      await exitRelayButton.click();
      await util.expectRoute(RoutePath.main);
    });
  });

  test.describe('Filter', () => {
    test.describe('Applied from filter view', () => {
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
          const wireguardRelays = {
            countries: relayList.countries.map(({ cities, ...country }) => ({
              ...country,
              cities: cities.map(({ relays, ...city }) => ({
                ...city,
                relays,
              })),
            })),
          };

          // Select one provider
          const provider = wireguardRelays.countries[0].cities[0].relays[0].provider;
          await routes.filter.checkProviderCheckbox(provider);

          await helpers.updateMockRelayFilter({
            providers: [provider],
          });

          await routes.filter.applyFilter();
          await util.expectRoute(RoutePath.selectLocation);
          const providerFilterChip = routes.selectLocation.selectors.filterChip('Providers: 1');
          await expect(providerFilterChip).toBeVisible();

          const relaySelectionPaths = helpers.toSelectionPaths(
            helpers.getRelaysByProvider(wireguardRelays, provider),
          );
          const relays = relaySelectionPaths.map((locatedRelay) => locatedRelay.relay);
          const relayNames = relays.map((relay) => relay.hostname);

          // Expand all accordions
          await helpers.expandLocatedRelays(relaySelectionPaths);

          const buttons = routes.selectLocation.selectors.relaysMatching(relayNames);

          // Expect all filtered relays to have a button
          await expect(buttons).toHaveCount(relays.length);

          // Clear filter
          await providerFilterChip.click();

          // Get all relays and expand accordions
          const allLocatedRelays = helpers.toSelectionPaths(relayList);
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
          await util.expectRoute(RoutePath.selectLocation);

          const ownerFilterChip = routes.selectLocation.selectors.filterChip('Rented');
          await expect(ownerFilterChip).toBeVisible();

          const relaySelectionPaths = helpers.toSelectionPaths(
            helpers.getRelaysByOwner(relayList, false),
          );
          const relays = relaySelectionPaths.map((locatedRelay) => locatedRelay.relay);
          const relayNames = relays.map((relay) => relay.hostname);

          // Expand all accordions
          await helpers.expandLocatedRelays(relaySelectionPaths);

          const buttons = routes.selectLocation.selectors.relaysMatching(relayNames);

          // Expect all filtered relays to have a button
          await expect(buttons).toHaveCount(relays.length);

          // Clear filter
          await ownerFilterChip.click();

          // Get all relays and expand accordions
          const allLocatedRelays = helpers.toSelectionPaths(relayList);
          await helpers.expandLocatedRelays(allLocatedRelays);

          // Should not have same length as all relays
          await expect(buttons).not.toHaveCount(allLocatedRelays.length);
        });
      });
    });
    test.describe('Filter by obfuscation', () => {
      test('Should apply filter when QUIC obfuscation is selected', async () => {
        const settings = getDefaultSettings();
        if ('normal' in settings.relaySettings) {
          settings.obfuscationSettings.selectedObfuscation = ObfuscationType.quic;
        }
        await util.ipc.settings[''].notify(settings);

        const relaySelectionPaths = helpers.toSelectionPaths(
          helpers.getRelaysByObfuscation(relayList, (relay) => 'quic' in relay),
        );
        const relays = relaySelectionPaths.map((locatedRelay) => locatedRelay.relay);
        const relayNames = relays.map((relay) => relay.hostname);

        await helpers.expandLocatedRelays(relaySelectionPaths);

        const buttons = routes.selectLocation.selectors.relaysMatching(relayNames);

        // Expect all filtered relays to have a button
        await expect(buttons).toHaveCount(relays.length);
      });

      test('Should apply filter when LWO obfuscation is selected', async () => {
        const settings = getDefaultSettings();
        if ('normal' in settings.relaySettings) {
          settings.obfuscationSettings.selectedObfuscation = ObfuscationType.lwo;
        }
        await util.ipc.settings[''].notify(settings);

        const relaySelectionPaths = helpers.toSelectionPaths(
          helpers.getRelaysByObfuscation(relayList, (relay) => relay.lwo),
        );
        const relays = relaySelectionPaths.map((locatedRelay) => locatedRelay.relay);
        const relayNames = relays.map((relay) => relay.hostname);

        await helpers.expandLocatedRelays(relaySelectionPaths);
        const buttons = routes.selectLocation.selectors.relaysMatching(relayNames);

        // Expect all filtered relays to have a button
        await expect(buttons).toHaveCount(relays.length);
      });
    });
  });
});
