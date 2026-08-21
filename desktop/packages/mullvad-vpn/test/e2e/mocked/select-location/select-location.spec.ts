import { expect, test } from '@playwright/test';
import { Page } from 'playwright';

import { getDefaultSettings } from '../../../../src/main/default-settings';
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

  test.describe('Multihop', () => {
    let initialSettings: ISettings = getDefaultSettings();

    test.beforeAll(async () => {
      initialSettings = await helpers.mockSettings({
        multihop: 'always',
      });
    });

    test.beforeEach(async () => {
      await routes.selectLocation.getEntryInput().click();
    });

    test('App should show entry selection', async () => {
      const entryInput = routes.selectLocation.getEntryInput();
      await expect(entryInput).toBeFocused();

      const locations = routes.selectLocation.getLocationsInAllLocations();
      expect(await locations.count()).toBeGreaterThan(0);
    });

    test('App should show exit selection', async () => {
      const exitInput = routes.selectLocation.getExitInput();
      await exitInput.click();
      await expect(exitInput).toBeFocused();

      const locations = routes.selectLocation.getLocationsInAllLocations();
      expect(await locations.count()).toBeGreaterThan(0);
    });

    test('Should show only wireguard servers in entry list', async () => {
      const wireguardRelays = relayList.countries[0].cities[0].relays;
      const hostnames = wireguardRelays.map((relay) => relay.hostname);
      const relaySelectionPaths = helpers.toSelectionPaths(
        helpers.getRelaysByHostnames(relayList, hostnames),
      );

      await helpers.expandLocatedRelays(relaySelectionPaths);

      const buttons = routes.selectLocation.getLocationsMatching(hostnames);
      await expect(buttons).toHaveCount(wireguardRelays.length);
    });

    test('Should show only wireguard servers in exit list', async () => {
      const exitInput = routes.selectLocation.getExitInput();
      await exitInput.click();

      const wireguardRelays = relayList.countries[0].cities[0].relays;
      const hostnames = wireguardRelays.map((relay) => relay.hostname);
      const relaySelectionPaths = helpers.toSelectionPaths(
        helpers.getRelaysByHostnames(relayList, hostnames),
      );

      await helpers.expandLocatedRelays(relaySelectionPaths);

      const buttons = routes.selectLocation.getLocationsMatching(hostnames);
      await expect(buttons).toHaveCount(wireguardRelays.length);
    });

    test('Should disable entry server in exit list', async () => {
      // Go to exit selection
      const exitInput = routes.selectLocation.getExitInput();
      await exitInput.click();

      // Set entry location to first relay in relay list
      const firstHostname = relayList.countries[0].cities[0].relays[0].hostname;
      const firstRelaySelectionPath = helpers.toSelectionPaths(
        helpers.getRelaysByHostnames(relayList, [firstHostname]),
      )[0];
      await helpers.mockEntryLocation(firstRelaySelectionPath, initialSettings);

      // Find same location in exit list and check that it is disabled
      await helpers.expandLocatedRelays([firstRelaySelectionPath]);
      const exitRelay = routes.selectLocation.getLocationsMatching([firstHostname]).first();

      await expect(exitRelay).toBeDisabled();
    });

    test('Should disable exit server in entry list', async () => {
      // Set exit location to first relay in relay list
      const firstHostname = relayList.countries[0].cities[0].relays[0].hostname;
      const firstRelaySelectionPath = helpers.toSelectionPaths(
        helpers.getRelaysByHostnames(relayList, [firstHostname]),
      )[0];
      await helpers.mockExitLocation(firstRelaySelectionPath, initialSettings);

      // Find same location in entry list and check that it is disabled
      await helpers.expandLocatedRelays([firstRelaySelectionPath]);
      const entryRelay = routes.selectLocation.getLocationsMatching([firstHostname]).first();

      await expect(entryRelay).toBeDisabled();
    });
  });

  test.describe('Recents', () => {
    let initialSettings: ISettings = getDefaultSettings();

    test.beforeEach(async () => {
      const settings = await helpers.mockRecents(recents);
      initialSettings = await helpers.mockCustomLists(customLists, settings);
    });

    test('Should show empty recent section when enabled and no recents', async () => {
      await helpers.mockRecents({
        entries: [],
        exits: [],
      });

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
      const geographicalExits = recents.exits.filter((recent) => !recent.customList);
      await helpers.mockRecents({ exits: geographicalExits, entries: [] });

      const recentLocations = routes.selectLocation.getLocationsInRecents();
      await expect(recentLocations).toHaveCount(geographicalExits.length);
    });

    test('Should show custom lists in recents section', async () => {
      const customListExits = recents.exits.filter((recent) => recent.customList);
      const settings = await helpers.mockRecents({ exits: customListExits, entries: [] });
      await helpers.mockCustomLists(customLists, settings);

      const recentLocations = routes.selectLocation.getLocationsInRecents();
      await expect(recentLocations).toHaveCount(customListExits.length);
    });

    test('Should show recents section for exits when recents is enabled', async () => {
      await helpers.mockSettings(
        {
          multihop: 'never',
        },
        initialSettings,
      );

      const recentLocations = routes.selectLocation.getLocationsInRecents();
      await expect(recentLocations).toHaveCount(recents.exits.length);
    });

    test('Should show recents section for entries when recents is enabled', async () => {
      await helpers.mockSettings(
        {
          multihop: 'always',
        },
        initialSettings,
      );

      const recentLocations = routes.selectLocation.getLocationsInRecents();
      await expect(recentLocations).toHaveCount(recents.exits.length);

      await routes.selectLocation.getExitInput().click();
      await expect(recentLocations).toHaveCount(recents.entries.length);
    });

    test('Should be able to add recent geographical location to custom list', async () => {
      const settings = await helpers.mockCustomLists(customLists, initialSettings);

      await helpers.mockSettings(
        {
          multihop: 'never',
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
      // Make sure menu transition has finished before clicking outside to close it
      await page.waitForTimeout(200);

      await page.mouse.click(0, 0); // Click outside to close menu
      await expect(addToCustomListButton).not.toBeVisible();
    });

    test('Should be able to edit or delete recent custom list', async () => {
      const settings = await helpers.mockCustomLists(customLists, initialSettings);

      await helpers.mockSettings(
        {
          multihop: 'never',
        },
        settings,
      );

      const customListName = customLists[0].name;
      await routes.selectLocation.getRecentMenuButton(customListName).click();
      const editCustomListButton = routes.selectLocation.getEditCustomListButton();
      await expect(editCustomListButton).toBeVisible();

      const deleteCustomListButton = routes.selectLocation.getDeleteCustomListButton();
      await expect(deleteCustomListButton).toBeVisible();
      // Make sure menu transition has finished before clicking outside to close it
      await page.waitForTimeout(200);

      await page.mouse.click(0, 0); // Click outside to close menu
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

          const buttons = routes.selectLocation.getLocationsMatching(relayNames);

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

          const buttons = routes.selectLocation.getLocationsMatching(relayNames);

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

        const buttons = routes.selectLocation.getLocationsMatching(relayNames);

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
        const buttons = routes.selectLocation.getLocationsMatching(relayNames);

        // Expect all filtered relays to have a button
        await expect(buttons).toHaveCount(relays.length);
      });
    });
  });
});
