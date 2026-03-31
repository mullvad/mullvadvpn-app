import { expect, test } from '@playwright/test';
import { Page } from 'playwright';

import { getDefaultSettings } from '../../../../src/main/default-settings';
import { colorTokens } from '../../../../src/renderer/lib/foundations';
import {
  type ISettings,
  ObfuscationType,
  Ownership,
} from '../../../../src/shared/daemon-rpc-types';
import { RoutePath } from '../../../../src/shared/routes';
import { mockData } from '../../mock-data';
import { RoutesObjectModel } from '../../route-object-models';
import { MockedTestUtils, startMockedApp } from '../mocked-utils';
import { createHelpers, SelectLocationHelpers } from './helpers';

const { relayList, customLists, recents } = mockData;

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
    const input = routes.selectLocation.getSearchInput();
    await expect(input).toBeFocused();
  });

  test.describe('Multihop enabled', () => {
    test.beforeAll(async () => {
      await helpers.updateMockSettings({
        multihop: true,
      });
    });

    test.beforeEach(async () => {
      await routes.selectLocation.getEntryButton().click();
    });

    test('App should show entry selection', async () => {
      const entryButton = routes.selectLocation.getEntryButton();
      await expect(entryButton).toHaveCSS('background-color', colorTokens.green);

      const locations = routes.selectLocation.getLocationsInAllLocations();
      expect(await locations.count()).toBeGreaterThan(0);
    });

    test('App should show exit selection', async () => {
      const exitButton = routes.selectLocation.getExitButton();
      await exitButton.click();
      await expect(exitButton).toHaveCSS('background-color', colorTokens.green);

      const locations = routes.selectLocation.getLocationsInAllLocations();
      expect(await locations.count()).toBeGreaterThan(0);
    });

    test("App shouldn't show entry selection when daita is enabled without direct only", async () => {
      await helpers.updateMockSettings({
        multihop: true,
        daita: true,
        directOnly: false,
      });

      const entryButton = routes.selectLocation.getEntryButton();
      await expect(entryButton).toHaveCSS('background-color', colorTokens.green);

      const locations = routes.selectLocation.getLocationsInAllLocations();
      await expect(locations).toHaveCount(0);
    });

    test('App should show entry selection when daita is enabled with direct only', async () => {
      await helpers.updateMockSettings({
        multihop: true,
        daita: true,
        directOnly: true,
      });

      const entryButton = routes.selectLocation.getEntryButton();
      await expect(entryButton).toHaveCSS('background-color', colorTokens.green);

      const locations = routes.selectLocation.getLocationsInAllLocations();
      expect(await locations.count()).toBeGreaterThan(0);
    });

    test('Should show only wireguard servers in entry list', async () => {
      const entryButton = routes.selectLocation.getEntryButton();
      await entryButton.click();

      const wireguardRelays = relayList.countries[0].cities[0].relays;
      const hostnames = wireguardRelays.map((relay) => relay.hostname);
      const relaySelectionPaths = helpers.toSelectionPaths(
        helpers.getRelaysByHostnames(relayList, hostnames),
      );

      await helpers.expandLocatedRelays(relaySelectionPaths);

      const buttons = routes.selectLocation.getRelaysMatching(hostnames);
      await expect(buttons).toHaveCount(wireguardRelays.length);
    });

    test('Should show only wireguard servers in exit list', async () => {
      const exitButton = routes.selectLocation.getExitButton();
      await exitButton.click();

      const wireguardRelays = relayList.countries[0].cities[0].relays;
      const hostnames = wireguardRelays.map((relay) => relay.hostname);
      const relaySelectionPaths = helpers.toSelectionPaths(
        helpers.getRelaysByHostnames(relayList, hostnames),
      );

      await helpers.expandLocatedRelays(relaySelectionPaths);

      const buttons = routes.selectLocation.getRelaysMatching(hostnames);
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

      const entryButton = routes.selectLocation.getEntryButton();
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

      await routes.selectLocation.getRelaysMatching([entryRelay.hostname]).first().click();

      await helpers.updateEntryLocation(relaySelectionPaths[0], settings);
      await helpers.expandLocatedRelays(relaySelectionPaths);
      const entryRelayButton = routes.selectLocation.getRelaysMatching([entryRelay.hostname]);
      await expect(entryRelayButton).toBeDisabled();

      const relaySelectionPathsExit = helpers.toSelectionPaths(
        helpers.getRelaysByHostnames(relayList, [exitRelay.hostname]),
      );
      await helpers.expandLocatedRelays(relaySelectionPathsExit);

      // Clicking exit relay should navigate to main route
      const exitRelayButton = routes.selectLocation.getRelaysMatching([exitRelay.hostname]);
      await exitRelayButton.click();
      await util.expectRoute(RoutePath.main);
    });
  });

  test.describe('Recents', () => {
    let initialSettings: ISettings = getDefaultSettings();

    test.beforeEach(async () => {
      const settings = await helpers.mockRecents(recents);
      initialSettings = await helpers.mockCustomLists(customLists, settings);
    });

    test('Should show empty recent section when enabled and no recents', async () => {
      await helpers.mockRecents([]);

      const recentSection = routes.selectLocation.getRecentsSection();
      await expect(recentSection).toBeVisible();

      const recentLocations = routes.selectLocation.getLocationsInLocator(recentSection);
      await expect(recentLocations).toHaveCount(0);
    });

    test('Should not show recents section when recents is disabled', async () => {
      await helpers.mockRecents(undefined);

      const recentSection = routes.selectLocation.getRecentsSection();
      await expect(recentSection).toBeHidden();
    });

    test('Should show geographical locations in recents section', async () => {
      const singlehopRecents = recents.filter(
        (recent) => recent.type === 'singlehop' && !recent.location.customList,
      );
      await helpers.mockRecents(singlehopRecents);

      const recentLocations = routes.selectLocation.getLocationsInRecents();
      await expect(recentLocations).toHaveCount(singlehopRecents.length);
    });

    test('Should show custom lists in recents section', async () => {
      const customListRecents = recents.filter(
        (recent) => recent.type === 'singlehop' && recent.location.customList,
      );
      const settings = await helpers.mockRecents(customListRecents);
      await helpers.mockCustomLists(customLists, settings);

      const recentLocations = routes.selectLocation.getLocationsInRecents();
      await expect(recentLocations).toHaveCount(customListRecents.length);
    });

    test('Should show recents section when recents is enabled and using singlehop', async () => {
      const singlehopRecent = recents.find((recent) => recent.type === 'singlehop');
      if (!singlehopRecent) {
        throw new Error('No singlehop recent found in mocked data');
      }

      await helpers.updateMockSettings(
        {
          multihop: false,
        },
        initialSettings,
      );

      const singlehopRecents = recents.filter((recent) => recent.type === 'singlehop');
      const recentLocations = routes.selectLocation.getLocationsInRecents();
      await expect(recentLocations).toHaveCount(singlehopRecents.length);
    });

    test('Should show recents section when recents is enabled and using multihop', async () => {
      const multihopRecent = recents.find((recent) => recent.type === 'multihop');
      if (!multihopRecent) {
        throw new Error('No multihop recent found in mocked data');
      }

      await helpers.updateMockSettings(
        {
          multihop: true,
        },
        initialSettings,
      );

      const multihopRecents = recents.filter((recent) => recent.type === 'multihop');
      const recentLocations = routes.selectLocation.getLocationsInRecents();

      await expect(recentLocations).toHaveCount(multihopRecents.length);

      await routes.selectLocation.getEntryButton().click();

      await expect(recentLocations).toHaveCount(multihopRecents.length);
    });

    test('Should be able to add recent geographical location to custom list', async () => {
      const singlehopRecent = recents.find((recent) => recent.type === 'singlehop');
      if (!singlehopRecent) {
        throw new Error('No singlehop recent found in mocked data');
      }
      const settings = await helpers.mockCustomLists(customLists, initialSettings);

      await helpers.updateMockSettings(
        {
          multihop: false,
        },
        settings,
      );

      const recentLocations = routes.selectLocation.getLocationsInRecents();
      const firstRecent = recentLocations.first();
      const firstRecentName = await firstRecent.innerText();

      await routes.selectLocation.getRecentMenuButton(firstRecentName).click();

      const customListName = customLists[0].name;
      const addToCustomListButton = routes.selectLocation.getAddToCustomListButton(
        firstRecentName,
        customListName,
      );
      await expect(addToCustomListButton).toBeVisible();

      const addToNewCustomListButton =
        routes.selectLocation.getAddToNewCustomListButton(firstRecentName);
      await expect(addToNewCustomListButton).toBeVisible();

      await page.locator('body').click(); // Click outside to close menu
      await expect(addToCustomListButton).not.toBeVisible();
    });

    test('Should be able to edit or delete recent custom list', async () => {
      const recentCustomList = recents.find(
        (recent) => recent.type === 'singlehop' && recent.location.customList,
      );
      if (!recentCustomList) {
        throw new Error('No recent custom list found in mocked data');
      }
      const settings = await helpers.mockCustomLists(customLists, initialSettings);

      await helpers.updateMockSettings(
        {
          multihop: false,
        },
        settings,
      );

      const customListName = customLists[0].name;
      await routes.selectLocation.getRecentMenuButton(customListName).click();
      const editCustomListButton = routes.selectLocation.getEditCustomListButton();
      await expect(editCustomListButton).toBeVisible();

      const deleteCustomListButton = routes.selectLocation.getDeleteCustomListButton();
      await expect(deleteCustomListButton).toBeVisible();

      await page.locator('body').click(); // Click outside to close menu
      await expect(deleteCustomListButton).not.toBeVisible();
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

          await Promise.all([
            util.ipc.settings.setRelaySettings.handle(),
            routes.filter.applyFilter(),
          ]);
          await helpers.updateMockRelayFilter({
            providers: [provider],
          });
          await util.expectRoute(RoutePath.selectLocation);
          const providerFilterChip = routes.selectLocation.getFilterChip('Providers: 1');
          await expect(providerFilterChip).toBeVisible();

          const relaySelectionPaths = helpers.toSelectionPaths(
            helpers.getRelaysByProvider(wireguardRelays, provider),
          );
          const relays = relaySelectionPaths.map((locatedRelay) => locatedRelay.relay);
          const relayNames = relays.map((relay) => relay.hostname);

          // Expand all accordions
          await helpers.expandLocatedRelays(relaySelectionPaths);

          const buttons = routes.selectLocation.getRelaysMatching(relayNames);

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

          const ownerFilterChip = routes.selectLocation.getFilterChip('Rented');
          await expect(ownerFilterChip).toBeVisible();

          const relaySelectionPaths = helpers.toSelectionPaths(
            helpers.getRelaysByOwner(relayList, false),
          );
          const relays = relaySelectionPaths.map((locatedRelay) => locatedRelay.relay);
          const relayNames = relays.map((relay) => relay.hostname);

          // Expand all accordions
          await helpers.expandLocatedRelays(relaySelectionPaths);

          const buttons = routes.selectLocation.getRelaysMatching(relayNames);

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

        const buttons = routes.selectLocation.getRelaysMatching(relayNames);

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
        const buttons = routes.selectLocation.getRelaysMatching(relayNames);

        // Expect all filtered relays to have a button
        await expect(buttons).toHaveCount(relays.length);
      });
    });
  });
});
