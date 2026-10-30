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
    let settings: ISettings = getDefaultSettings();

    test.beforeEach(async () => {
      settings = await helpers.mockSettings({
        multihop: 'always',
      });
      await routes.selectLocation.gotoEntryLocations();
    });

    test('App should show entry selection', async () => {
      const entryInput = routes.selectLocation.getEntryInput();
      await expect(entryInput).toBeFocused();

      const locations = routes.selectLocation.getLocationsInAllLocations();
      expect(await locations.count()).toBeGreaterThan(0);
    });

    test('App should show exit selection', async () => {
      await routes.selectLocation.gotoExitLocations();
      const exitInput = routes.selectLocation.getExitInput();
      await expect(exitInput).toBeFocused();

      const locations = routes.selectLocation.getLocationsInAllLocations();
      expect(await locations.count()).toBeGreaterThan(0);
    });

    test('Should show relays and automatic location in entry list', async () => {
      const relays = relayList.countries[0].cities[0].relays;
      const hostnames = relays.map((relay) => relay.hostname);
      const relaySelectionPaths = helpers.toSelectionPaths(
        helpers.getRelaysByHostnames(relayList, hostnames),
      );

      await helpers.expandLocatedRelays(relaySelectionPaths);

      const buttons = routes.selectLocation.getLocationsMatching(hostnames);
      await expect(buttons).toHaveCount(relays.length);

      const automaticLocation = routes.selectLocation.getAutomaticLocation();
      await expect(automaticLocation).toBeVisible();
    });

    test('Should update to automatic location after confirming entry filters override warning dialog', async () => {
      const providers = helpers.getProvidersFromRelayList(relayList);
      settings = await helpers.updateMockEntryRelayFilter(
        {
          providers: [providers[0]],
        },
        settings,
      );
      settings = await helpers.mockSettings(
        {
          entryLocation: {
            only: {
              country: relayList.countries[0].code,
            },
          },
        },
        settings,
      );

      const automaticLocation = routes.selectLocation.getAutomaticLocation();
      await automaticLocation.click();

      const automaticLocationWarningDialogConfirmButton =
        routes.selectLocation.getAutomaticLocationWarningDialogContinueButton();
      await expect(automaticLocationWarningDialogConfirmButton).toBeVisible();

      await Promise.all([
        util.ipc.settings.setRelaySettings.expect(),
        automaticLocationWarningDialogConfirmButton.click(),
      ]);
    });

    test('Should show relays in exit list', async () => {
      await routes.selectLocation.gotoExitLocations();

      const relays = relayList.countries[0].cities[0].relays;
      const hostnames = relays.map((relay) => relay.hostname);
      const relaySelectionPaths = helpers.toSelectionPaths(
        helpers.getRelaysByHostnames(relayList, hostnames),
      );

      await helpers.expandLocatedRelays(relaySelectionPaths);

      const buttons = routes.selectLocation.getLocationsMatching(hostnames);
      await expect(buttons).toHaveCount(relays.length);
    });

    test('Should disable entry server in exit list', async () => {
      // Go to exit selection
      await routes.selectLocation.gotoExitLocations();

      // Set entry location to first relay in relay list
      const firstHostname = relayList.countries[0].cities[0].relays[0].hostname;
      const firstRelaySelectionPath = helpers.toSelectionPaths(
        helpers.getRelaysByHostnames(relayList, [firstHostname]),
      )[0];
      settings = await helpers.mockEntryLocation(firstRelaySelectionPath, settings);

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
      settings = await helpers.mockExitLocation(firstRelaySelectionPath, settings);

      // Find same location in entry list and check that it is disabled
      await helpers.expandLocatedRelays([firstRelaySelectionPath]);
      const entryRelay = routes.selectLocation.getLocationsMatching([firstHostname]).first();

      await expect(entryRelay).toBeDisabled();
    });
  });

  test.describe('Location selector', () => {
    let settings = getDefaultSettings();

    test.beforeEach(async () => {
      settings = getDefaultSettings();
      settings = await helpers.mockSettings(
        {
          multihop: 'always',
        },
        settings,
      );
      await routes.selectLocation.gotoExitLocations();
    });

    test('Should display entry and exit when multihop mode is "always"', async () => {
      const entryInput = routes.selectLocation.getEntryInput();
      const exitInput = routes.selectLocation.getExitInput();

      await expect(entryInput).toBeVisible();
      await expect(exitInput).toBeVisible();
    });

    test('Should only display exit when multihop mode is "never"', async () => {
      settings = await helpers.mockSettings(
        {
          multihop: 'never',
        },
        settings,
      );

      const entryInput = routes.selectLocation.getEntryInput();
      const exitInput = routes.selectLocation.getExitInput();

      await expect(entryInput).toBeHidden();
      await expect(exitInput).toBeVisible();
    });

    test('Should display only exit when multihop mode is "when-needed" and has no automatic entry', async () => {
      settings = await helpers.mockSettings(
        {
          multihop: 'when-needed',
        },
        settings,
      );

      const entryInput = routes.selectLocation.getEntryInput();
      const exitInput = routes.selectLocation.getExitInput();

      await expect(entryInput).toBeHidden();
      await expect(exitInput).toBeVisible();
    });

    test('Should display entry and exit when multihop mode is "when-needed" and has automatic entry', async () => {
      // Need to set up all the circumstances that makes the Automatic entry appear
      settings = await helpers.mockSettings(
        {
          multihop: 'when-needed',
        },
        settings,
      );

      await helpers.mockConnectTunnel({
        ...mockData.location,
        entryHostname: relayList.countries[0].cities[0].relays[0].hostname,
      });

      await helpers.mockEntryAndExitLocations(
        {
          entryLocation: {
            only: {
              country: relayList.countries[0].code,
            },
          },
          exitLocation: {
            only: {
              country: relayList.countries[0].code,
            },
          },
        },
        settings,
      );

      const entryAutomaticButton = routes.selectLocation.getEntryAutomaticButton(
        relayList.countries[0].name,
      );
      const exitInput = routes.selectLocation.getExitInput();

      await expect(entryAutomaticButton).toBeVisible();
      await expect(exitInput).toBeVisible();
    });

    test('Should switch multihop mode to "always" when clicking button in automatic entry selected view', async () => {
      // Need to set up all the circumstances that makes the Automatic entry appear
      settings = await helpers.mockSettings(
        {
          multihop: 'when-needed',
        },
        settings,
      );

      settings = await helpers.mockEntryAndExitLocations(
        {
          entryLocation: {
            only: {
              country: relayList.countries[0].code,
            },
          },
          exitLocation: {
            only: {
              country: relayList.countries[0].code,
            },
          },
        },
        settings,
      );

      await helpers.mockConnectTunnel({
        ...mockData.location,
        entryHostname: relayList.countries[0].cities[0].relays[0].hostname,
      });

      // Then we can click the button to go to automatic entry view
      await routes.selectLocation.gotoEntryAutomatic(relayList.countries[0].name);

      // Then we can click the button to update multihop mode
      await Promise.all([
        util.ipc.settings.setRelaySettings.expect(),
        routes.selectLocation.getAutomaticEntryMultihopModeToAlwaysButton().click(),
      ]);
    });
  });

  test.describe('Recents', () => {
    let settings: ISettings = getDefaultSettings();

    test.beforeEach(async () => {
      settings = await helpers.mockRecents(recents, settings);
      settings = await helpers.mockCustomLists(customLists, settings);
      settings = await helpers.mockSettings(
        {
          multihop: 'always',
        },
        settings,
      );
      await routes.selectLocation.gotoExitLocations();
    });

    test('Should show empty recent section when enabled and no recents', async () => {
      settings = await helpers.mockRecents(
        {
          entries: [],
          exits: [],
        },
        settings,
      );

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
      settings = await helpers.mockSettings(
        {
          multihop: 'never',
        },
        settings,
      );

      const recentLocations = routes.selectLocation.getLocationsInRecents();
      await expect(recentLocations).toHaveCount(recents.exits.length);
    });

    test('Should show recents section for entries when recents is enabled', async () => {
      settings = await helpers.mockSettings(
        {
          multihop: 'always',
        },
        settings,
      );

      const recentLocations = routes.selectLocation.getLocationsInRecents();
      await expect(recentLocations).toHaveCount(recents.exits.length);

      await routes.selectLocation.gotoEntryLocations();
      await expect(recentLocations).toHaveCount(recents.entries.length);
    });

    test('Should display automatic location in recents', async () => {
      await routes.selectLocation.gotoEntryLocations();
      const recentLocations = routes.selectLocation.getLocationsInRecents();
      const automaticLocation = recentLocations.filter({ hasText: 'Automatic' });

      await expect(automaticLocation).toHaveCount(1);
    });

    test('Should be able to add recent geographical location to custom list', async () => {
      settings = await helpers.mockCustomLists(customLists, settings);

      settings = await helpers.mockSettings(
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
      settings = await helpers.mockCustomLists(customLists, settings);

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
    test.describe('Filter view', () => {
      let settings = getDefaultSettings();

      test.beforeEach(async () => {
        settings = getDefaultSettings();
        await helpers.resetToSelectLocationView();
      });

      const testCases = [
        {
          title: 'Exit filter',
          locationType: 'exit',
        },
        {
          title: 'Entry filter',
          locationType: 'entry',
        },
      ] as const;
      testCases.forEach(({ title, locationType }) => {
        test.describe(`${title}`, () => {
          test.beforeEach(async () => {
            // Use multihop: always to ensure both entry and exit filter buttons are visible
            settings = await helpers.mockSettings(
              {
                multihop: 'always',
              },
              settings,
            );
            await helpers.resetToFilterView(locationType);
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
              const providers = helpers.getProvidersFromRelayList(relayList);
              const provider = providers[0];
              await routes.filter.checkProviderCheckbox(provider);

              await Promise.all([
                util.ipc.settings.setRelaySettings.handle(),
                routes.filter.applyFilter(),
              ]);
              settings = await helpers.updateMockRelayFilter(
                {
                  providers: [provider],
                  locationType,
                },
                settings,
              );
              await util.expectRoute(RoutePath.selectLocation);
              // TODO: Remove goToLocations when the Filter view selects
              // the appropriate exit or entry locations automatically
              await routes.selectLocation.gotoLocations(locationType);

              const providerFilterChip = routes.selectLocation.getFilterChipButton('Providers: 1');
              await expect(providerFilterChip).toBeVisible();

              // Clear filter
              await Promise.all([
                util.ipc.settings.setRelaySettings.handle(),
                providerFilterChip.click(),
              ]);
            });
          });

          test.describe('Filter by ownership', () => {
            test('Should apply filter when selecting ownership', async () => {
              // Select rented only
              await routes.filter.expandOwnership();
              await routes.filter.selectOwnershipOption('Rented only');
              settings = await helpers.updateMockRelayFilter(
                {
                  ownership: Ownership.rented,
                  locationType,
                },
                settings,
              );

              await routes.filter.applyFilter();
              await util.expectRoute(RoutePath.selectLocation);
              // TODO: Remove goToLocations when the Filter view selects
              // the appropriate exit or entry locations automatically
              await routes.selectLocation.gotoLocations(locationType);

              const ownerFilterChip = routes.selectLocation.getFilterChipButton('Rented');
              await expect(ownerFilterChip).toBeVisible();

              // Clear filter
              await Promise.all([
                util.ipc.settings.setRelaySettings.handle(),
                ownerFilterChip.click(),
              ]);
            });
          });

          test('Should display filter overridden info when using an automatic location', async () => {
            // Need to set up all the circumstances that makes the Automatic entry appear
            settings = await helpers.mockSettings(
              {
                multihop: 'when-needed',
              },
              settings,
            );

            settings = await helpers.mockEntryAndExitLocations(
              {
                entryLocation: 'any',
                exitLocation: 'any',
              },
              settings,
            );

            await helpers.mockConnectTunnel({
              ...mockData.location,
              entryHostname: relayList.countries[0].cities[0].relays[0].hostname,
            });

            // Then we can verify that the info text appears.
            await expect(routes.filter.getFilterOverriddenText()).toBeVisible();

            // And verify that text is still present when using `multihop: always`
            settings = await helpers.mockSettings(
              {
                multihop: 'always',
              },
              settings,
            );

            await expect(routes.filter.getFilterOverriddenText()).toBeVisible();
          });
        });
      });
    });

    test.describe('multihop: never', () => {
      let settings = getDefaultSettings();

      test.beforeEach(async () => {
        settings = getDefaultSettings();
        settings = await helpers.mockSettings(
          {
            multihop: 'never',
          },
          settings,
        );
        await helpers.resetToSelectLocationView();
      });

      test('Should go to "Exit Filter" view when clicking exit locations filter button', async () => {
        await routes.selectLocation.gotoExitFilter();

        await expect(routes.filter.getHeading('Exit filter')).toBeVisible();
      });

      test('Should show filter chips for active VPN settings in "Exit Filter" view', async () => {
        await routes.selectLocation.gotoExitFilter();

        settings = await helpers.mockSettings(
          {
            multihop: 'never',
            daita: true,
          },
          settings,
        );

        const daitaFilterChip = routes.filter.getFilterChipLink('DAITA');
        await expect(daitaFilterChip).toBeVisible();

        settings = await helpers.mockAntiCensorship(ObfuscationType.quic, settings);

        const quicFilterChip = routes.filter.getFilterChipLink('Obfuscation: QUIC');
        await expect(quicFilterChip).toBeVisible();

        settings = await helpers.mockAntiCensorship(ObfuscationType.lwo, settings);

        const lwoFilterChip = routes.filter.getFilterChipLink('Obfuscation: LWO');
        await expect(lwoFilterChip).toBeVisible();
      });

      test('Should apply filter when QUIC when viewing locations', async () => {
        settings = await helpers.mockAntiCensorship(ObfuscationType.quic, settings);

        const quicFilterChip = routes.selectLocation.getFilterChipLink('Obfuscation: QUIC');
        await expect(quicFilterChip).toBeVisible();

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

      test('Should display LWO filter when it is selected when viewing locations', async () => {
        settings = await helpers.mockAntiCensorship(ObfuscationType.lwo, settings);

        const lwoFilterChip = routes.selectLocation.getFilterChipLink('Obfuscation: LWO');
        await expect(lwoFilterChip).toBeVisible();

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

      test('Should display locations filter when viewing locations', async () => {
        settings = await helpers.mockSettings(
          {
            multihop: 'always',
          },
          settings,
        );
        await routes.selectLocation.gotoExitLocations();

        const providers = helpers.getProvidersFromRelayList(relayList);
        settings = await helpers.updateMockExitRelayFilter(
          {
            providers: [providers[0]],
          },
          settings,
        );

        const providerFilterChip = routes.selectLocation.getFilterChipButton('Providers: 1');
        await expect(providerFilterChip).toBeVisible();

        settings = await helpers.updateMockExitRelayFilter(
          {
            ownership: Ownership.rented,
          },
          settings,
        );

        const ownerFilterChip = routes.selectLocation.getFilterChipButton('Rented');
        await expect(ownerFilterChip).toBeVisible();
      });
    });

    test.describe('multihop: always', () => {
      let settings = getDefaultSettings();

      test.beforeEach(async () => {
        settings = getDefaultSettings();
        settings = await helpers.mockSettings(
          {
            multihop: 'always',
          },
          settings,
        );
        await helpers.resetToSelectLocationView();
      });

      test('Should go to "Exit Filter" view when clicking exit locations filter button', async () => {
        await routes.selectLocation.gotoExitFilter();

        await expect(routes.filter.getHeading('Exit filter')).toBeVisible();
      });

      test('Should go to "Entry Filter" view when clicking entry locations filter button', async () => {
        await routes.selectLocation.gotoEntryFilter();

        await expect(routes.filter.getHeading('Entry filter')).toBeVisible();
      });

      test('Should display locations exit filter when viewing exit locations', async () => {
        settings = await helpers.mockSettings({
          multihop: 'always',
        });
        await routes.selectLocation.gotoExitLocations();

        const providers = helpers.getProvidersFromRelayList(relayList);
        await helpers.updateMockExitRelayFilter(
          {
            providers: [providers[0]],
          },
          settings,
        );

        const providerFilterChip = routes.selectLocation.getFilterChipButton('Providers: 1');
        await expect(providerFilterChip).toBeVisible();

        settings = await helpers.updateMockExitRelayFilter(
          {
            ownership: Ownership.rented,
          },
          settings,
        );

        const ownerFilterChip = routes.selectLocation.getFilterChipButton('Rented');
        await expect(ownerFilterChip).toBeVisible();
      });

      test('Should display locations entry filter when viewing entry locations', async () => {
        settings = await helpers.mockSettings({
          multihop: 'always',
        });
        await routes.selectLocation.gotoEntryLocations();

        const providers = helpers.getProvidersFromRelayList(relayList);
        settings = await helpers.updateMockEntryRelayFilter(
          {
            providers: [providers[0]],
          },
          settings,
        );

        const providerFilterChip = routes.selectLocation.getFilterChipButton('Providers: 1');
        await expect(providerFilterChip).toBeVisible();

        await helpers.updateMockEntryRelayFilter(
          {
            ownership: Ownership.rented,
          },
          settings,
        );

        const ownerFilterChip = routes.selectLocation.getFilterChipButton('Rented');
        await expect(ownerFilterChip).toBeVisible();
      });

      test('Should display separate locations filters when viewing entry and exit locations', async () => {
        let initialSettings: ISettings = getDefaultSettings();
        initialSettings = await helpers.mockSettings({
          multihop: 'always',
        });

        // Entry filters
        await routes.selectLocation.gotoEntryLocations();
        const providers = helpers.getProvidersFromRelayList(relayList);
        initialSettings = await helpers.updateMockEntryRelayFilter(
          {
            providers: [providers[0]],
          },
          initialSettings,
        );

        const providerFilterChip = routes.selectLocation.getFilterChipButton('Providers: 1');
        await expect(providerFilterChip).toBeVisible();

        // Exit filters
        await routes.selectLocation.gotoExitLocations();

        await helpers.updateMockExitRelayFilter(
          {
            providers: [],
          },
          initialSettings,
        );

        await expect(providerFilterChip).not.toBeVisible();
      });

      test('Should display QUIC filter when viewing entry locations and not when viewing exit locations', async () => {
        settings = await helpers.mockAntiCensorship(ObfuscationType.quic, settings);

        const quicFilterChip = routes.selectLocation.getFilterChipLink('Obfuscation: QUIC');

        // Present on entry
        await routes.selectLocation.gotoEntryLocations();
        await expect(quicFilterChip).toBeVisible();

        // Not present on exit
        await routes.selectLocation.gotoExitLocations();
        await expect(quicFilterChip).not.toBeVisible();
      });

      test('Should display QUIC filter chip when viewing entry filters and not when viewing exit filters', async () => {
        settings = await helpers.mockAntiCensorship(ObfuscationType.quic, settings);

        const quicFilterChip = routes.filter.getFilterChipLink('Obfuscation: QUIC');

        // Present on entry
        await routes.selectLocation.gotoEntryFilter();
        await expect(quicFilterChip).toBeVisible();
        await routes.filter.gotoSelectLocation();

        // Not present on exit
        await routes.selectLocation.gotoExitFilter();
        await expect(quicFilterChip).not.toBeVisible();
      });

      test('Should show a warning dialog when changing multihop mode without automatic entry location and with active entry filters', async () => {
        settings = await helpers.mockSettings(
          {
            entryLocation: {
              only: {
                country: relayList.countries[0].code,
              },
            },
          },
          settings,
        );

        const providers = helpers.getProvidersFromRelayList(relayList);
        settings = await helpers.updateMockEntryRelayFilter(
          {
            providers: [providers[0]],
          },
          settings,
        );

        // Open menu and select `when-needed`
        await routes.selectLocation.openHeaderMenu();
        const headerMenuItem = routes.selectLocation.getHeaderMenuItem('When needed');

        await expect(headerMenuItem).toBeVisible();
        await Promise.all([util.ipc.settings.setRelaySettings.ignore(), headerMenuItem.click()]);

        // Cancel first
        const cancelButton = routes.selectLocation.getEntryFiltersOverrideDialogButton('Cancel');
        await expect(cancelButton).toBeVisible();
        await cancelButton.click();

        // Trigger dialog again
        await Promise.all([util.ipc.settings.setRelaySettings.ignore(), headerMenuItem.click()]);

        const continueButton =
          routes.selectLocation.getEntryFiltersOverrideDialogButton('Continue');
        await expect(continueButton).toBeVisible();

        // Multihop mode should now be updated
        await Promise.all([util.ipc.settings.setRelaySettings.ignore(), continueButton.click()]);
      });
    });

    test.describe('multihop: when-needed', () => {
      let settings = getDefaultSettings();

      test.beforeEach(async () => {
        await helpers.mockDisconnectTunnel(mockData.location);
        settings = getDefaultSettings();

        settings = await helpers.mockSettings(
          {
            multihop: 'when-needed',
          },
          settings,
        );
        await helpers.resetToSelectLocationView();
      });

      test('Should go to "Exit Filter" view when clicking exit locations filter button', async () => {
        await routes.selectLocation.gotoExitFilter();

        await expect(routes.filter.getHeading('Exit filter')).toBeVisible();
      });

      test('Should go to "Entry Filter" view when clicking automatic entry locations filter button', async () => {
        // Need to set up all the circumstances that makes the Automatic entry appear
        settings = await helpers.mockSettings(
          {
            multihop: 'when-needed',
          },
          settings,
        );

        settings = await helpers.mockEntryAndExitLocations(
          {
            entryLocation: {
              only: {
                country: relayList.countries[0].code,
              },
            },
            exitLocation: {
              only: {
                country: relayList.countries[0].code,
              },
            },
          },
          settings,
        );

        await helpers.mockConnectTunnel({
          ...mockData.location,
          entryHostname: relayList.countries[0].cities[0].relays[0].hostname,
        });

        // Then we can click the button to go to the filter
        await routes.selectLocation.gotoEntryAutomaticFilter();

        await expect(routes.filter.getHeading('Entry filter')).toBeVisible();
      });

      test('Should only display location filters when viewing Exit locations', async () => {
        await routes.selectLocation.gotoExitLocations();

        // Set multihop: never to ensure QUIC filter can be displayed so we can
        // verify that it has been applied
        settings = await helpers.mockSettings(
          {
            multihop: 'never',
          },
          settings,
        );

        // Enable QUIC
        settings = await helpers.mockAntiCensorship(ObfuscationType.quic, settings);

        // Ensure QUIC filter can be seen when using multihop: never
        const quicFilterChip = routes.selectLocation.getFilterChipLink('Obfuscation: QUIC');
        await expect(quicFilterChip).toBeVisible();

        // Switch to multihop: when-needed to ensure QUIC filter is hidden
        settings = await helpers.mockSettings(
          {
            multihop: 'when-needed',
          },
          settings,
        );
        await expect(quicFilterChip).not.toBeVisible();

        // Set up providers filter to ensure it is only the QUIC filter that is not shown
        const providers = helpers.getProvidersFromRelayList(relayList);
        await helpers.updateMockExitRelayFilter(
          {
            providers: [providers[0]],
          },
          settings,
        );

        const providerFilterChip = routes.selectLocation.getFilterChipButton('Providers: 1');
        await expect(providerFilterChip).toBeVisible();
      });

      test('Should display QUIC filter chip when viewing entry filters and not when viewing exit filters', async () => {
        // Need to set up all the circumstances that makes the Automatic entry appear
        settings = await helpers.mockSettings(
          {
            multihop: 'when-needed',
          },
          settings,
        );

        settings = await helpers.mockEntryAndExitLocations(
          {
            entryLocation: {
              only: {
                country: relayList.countries[0].code,
              },
            },
            exitLocation: {
              only: {
                country: relayList.countries[0].code,
              },
            },
          },
          settings,
        );

        await helpers.mockConnectTunnel({
          ...mockData.location,
          entryHostname: relayList.countries[0].cities[0].relays[0].hostname,
        });

        // Then we can set up the anti-censorship method and verify that it is present under the right conditions
        settings = await helpers.mockAntiCensorship(ObfuscationType.quic, settings);

        const quicFilterChip = routes.filter.getFilterChipLink('Obfuscation: QUIC');

        // Present on entry filters when using automatic location
        await routes.selectLocation.gotoEntryAutomaticFilter();
        await expect(quicFilterChip).toBeVisible();
        await routes.filter.gotoSelectLocation();

        // Not present on exit
        await routes.selectLocation.gotoExitFilter();
        await expect(quicFilterChip).not.toBeVisible();
      });
    });

    test.describe('Filter error messages', () => {
      let settings = getDefaultSettings();

      test.beforeEach(async () => {
        settings = getDefaultSettings();
        settings = await helpers.mockSettings(
          {
            multihop: 'always',
          },
          settings,
        );
        await helpers.resetToSelectLocationView();
      });

      test('Show error message when location does not match filter settings', async () => {
        settings = await helpers.updateMockEntryRelayFilter(
          {
            ownership: Ownership.rented,
          },
          settings,
        );

        const relays = helpers.getRelaysByOwner(relayList, false);
        settings = await helpers.mockEntryAndExitLocations(
          {
            entryLocation: 'any',
            exitLocation: {
              only: {
                country: relays.countries[0].name,
                city: relays.countries[0].cities[0].name,
                hostname: relays.countries[0].cities[0].relays[0].hostname,
              },
            },
          },
          settings,
        );

        await expect(routes.selectLocation.getNonMatchingFilterErrorMessage()).toBeVisible();
      });
    });
  });
});
