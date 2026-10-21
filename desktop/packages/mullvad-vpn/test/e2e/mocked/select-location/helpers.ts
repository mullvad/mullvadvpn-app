import { Page } from 'playwright';

import { getDefaultSettings } from '../../../../src/main/default-settings';
import { LocationType } from '../../../../src/renderer/features/locations/types';
import {
  type Constraint,
  type CustomLists,
  type ILocation,
  IRelayList,
  IRelayListCity,
  IRelayListCountry,
  IRelayListHostname,
  ISettings,
  MultihopMode,
  Ownership,
  type Recents,
  type RelayLocation,
} from '../../../../src/shared/daemon-rpc-types';
import { RoutePath } from '../../../../src/shared/routes';
import { RoutesObjectModel } from '../../route-object-models';
import { MockedTestUtils } from '../mocked-utils';

export type RelaySelectionPath = {
  country: IRelayListCountry;
  city: IRelayListCity;
  relay: IRelayListHostname;
};

export const createHelpers = (page: Page, routes: RoutesObjectModel, utils: MockedTestUtils) => {
  const areAllCheckboxesChecked = async () => {
    const checkboxes = page.getByRole('checkbox');
    const count = await checkboxes.count();
    for (let i = 0; i < count; i++) {
      const checkbox = checkboxes.nth(i);
      const checked = await checkbox.isChecked();
      if (!checked) {
        return false;
      }
    }
    return true;
  };

  const expandLocatedRelays = async (locatedRelays: RelaySelectionPath[]) => {
    for (const locatedRelay of locatedRelays) {
      await routes.selectLocation.expandAccordion(locatedRelay.country.name);
      await routes.selectLocation.expandAccordion(locatedRelay.city.name);
    }
  };

  const toSelectionPaths = (relayList: IRelayList): RelaySelectionPath[] => {
    return relayList.countries.flatMap((country) =>
      country.cities.flatMap((city) => city.relays.map((relay) => ({ country, city, relay }))),
    );
  };

  const getRelaysByHostnames = (relayList: IRelayList, hostnames: string[]): IRelayList => {
    return {
      countries: relayList.countries
        .map((country) => ({
          ...country,
          cities: country.cities
            .map((city) => ({
              ...city,
              relays: city.relays.filter((relay) => hostnames.includes(relay.hostname)),
            }))
            .filter((city) => city.relays.length > 0),
        }))
        .filter((country) => country.cities.length > 0),
    };
  };

  const getProvidersFromRelayList = (relayList: IRelayList) => {
    const providersList = relayList.countries.flatMap(({ cities }) =>
      cities.flatMap((city) => city.relays.flatMap((relay) => relay.provider)),
    );

    return [...new Set(providersList)];
  };

  const getRelaysByProvider = (relayList: IRelayList, provider: string): IRelayList => {
    return {
      countries: relayList.countries
        .map((country) => ({
          ...country,
          cities: country.cities
            .map((city) => ({
              ...city,
              relays: city.relays.filter((relay) => relay.provider === provider),
            }))
            .filter((city) => city.relays.length > 0),
        }))
        .filter((country) => country.cities.length > 0),
    };
  };

  const getRelaysByOwner = (relayList: IRelayList, owned: boolean): IRelayList => {
    return {
      countries: relayList.countries
        .map((country) => ({
          ...country,
          cities: country.cities
            .map((city) => ({
              ...city,
              relays: city.relays.filter((relay) => relay.owned === owned),
            }))
            .filter((city) => city.relays.length > 0),
        }))
        .filter((country) => country.cities.length > 0),
    };
  };

  const getRelaysByObfuscation = (
    relayList: IRelayList,
    relayCondition: (relay: IRelayListHostname) => boolean,
  ): IRelayList => {
    return {
      countries: relayList.countries
        .map((country) => ({
          ...country,
          cities: country.cities
            .map((city) => ({
              ...city,
              relays: city.relays.filter((relay) => relayCondition(relay)),
            }))
            .filter((city) => city.relays.length > 0),
        }))
        .filter((country) => country.cities.length > 0),
    };
  };

  const resetOwnership = async () => {
    await routes.filter.expandOwnership();
    await routes.filter.selectOwnershipOption('Any');
    await routes.filter.collapseOwnership();
  };

  const resetProviders = async () => {
    await routes.filter.expandProviders();
    const allCheckboxesChecked = await areAllCheckboxesChecked();
    if (!allCheckboxesChecked) {
      await routes.filter.checkAllProvidersCheckbox();
    }
    await routes.filter.collapseProviders();
  };

  const resetToFilterView = async (locationType: LocationType) => {
    const currentRoute = await utils.getCurrentRoute();
    if (currentRoute === RoutePath.selectLocation) {
      if (locationType === LocationType.exit) {
        await routes.selectLocation.gotoExitFilter();
      } else {
        await routes.selectLocation.gotoEntryFilter();
      }
    }
  };

  const resetToSelectLocationView = async () => {
    const currentRoute = await utils.getCurrentRoute();
    if (currentRoute === RoutePath.filter) {
      await routes.filter.gotoSelectLocation();
      // TODO: Remove gotoExitLocations when the Filter view selects
      // the appropriate exit or entry locations automatically
      await routes.selectLocation.gotoExitLocations();
    }
  };

  const updateMockRelayFilter = async (
    {
      ownership,
      providers,
      locationType,
    }: {
      locationType: LocationType;
      ownership?: Ownership;
      providers?: string[];
    },
    settings?: ISettings,
  ) => {
    if (!settings) {
      settings = getDefaultSettings();
    }

    if ('normal' in settings.relaySettings) {
      if (ownership) {
        if (locationType === LocationType.exit) {
          settings.relaySettings.normal.ownership = ownership;
        } else {
          settings.relaySettings.normal.wireguardConstraints.entryOwnership = ownership;
        }
      }
      if (providers) {
        if (locationType === LocationType.exit) {
          settings.relaySettings.normal.providers = providers;
        } else {
          settings.relaySettings.normal.wireguardConstraints.entryProviders = providers;
        }
      }
    }
    await utils.ipc.settings[''].notify(settings);

    return settings;
  };

  const updateMockEntryRelayFilter = async (
    {
      ownership,
      providers,
    }: {
      ownership?: Ownership;
      providers?: string[];
    },
    settings?: ISettings,
  ) => {
    return updateMockRelayFilter(
      {
        ownership,
        providers,
        locationType: LocationType.entry,
      },
      settings,
    );
  };

  const updateMockExitRelayFilter = async (
    {
      ownership,
      providers,
    }: {
      ownership?: Ownership;
      providers?: string[];
    },
    settings?: ISettings,
  ) => {
    return updateMockRelayFilter(
      {
        ownership,
        providers,
        locationType: LocationType.exit,
      },
      settings,
    );
  };

  const mockSettings = async (
    {
      daita,
      multihop,
      entryLocation,
    }: {
      multihop?: MultihopMode;
      daita?: boolean;
      entryLocation?: Constraint<RelayLocation>;
    },
    settings?: ISettings,
  ) => {
    if (!settings) {
      settings = getDefaultSettings();
    }
    if ('normal' in settings.relaySettings) {
      if (multihop !== undefined) {
        settings.relaySettings.normal.wireguardConstraints.multihop = multihop;
      }
      if (settings.tunnelOptions.daita) {
        if (daita !== undefined) {
          settings.tunnelOptions.daita = daita;
        }
      }
      if (entryLocation) {
        settings.relaySettings.normal.wireguardConstraints.entryLocation = entryLocation;
      }
    }

    await utils.ipc.settings[''].notify(settings);

    return settings;
  };

  const mockEntryLocation = async (relay: RelaySelectionPath, settings?: ISettings) => {
    if (!settings) {
      settings = getDefaultSettings();
    }

    if ('normal' in settings.relaySettings) {
      settings.relaySettings.normal.wireguardConstraints.entryLocation = {
        only: {
          hostname: relay.relay.hostname,
          country: relay.country.code,
          city: relay.city.code,
        },
      };
    }

    await utils.ipc.settings[''].notify(settings);

    return settings;
  };

  const mockExitLocation = async (relay: RelaySelectionPath, settings?: ISettings) => {
    if (!settings) {
      settings = getDefaultSettings();
    }

    if ('normal' in settings.relaySettings) {
      settings.relaySettings.normal.location = {
        only: {
          hostname: relay.relay.hostname,
          country: relay.country.code,
          city: relay.city.code,
        },
      };
    }

    await utils.ipc.settings[''].notify(settings);

    return settings;
  };

  const mockCustomLists = async (customLists: CustomLists, settings?: ISettings) => {
    if (!settings) {
      settings = getDefaultSettings();
    }

    const newSettings: ISettings = {
      ...settings,
      customLists,
    };
    await utils.ipc.settings[''].notify(newSettings);

    return newSettings;
  };

  const mockRecents = async (recents?: Recents, settings?: ISettings) => {
    if (!settings) {
      settings = getDefaultSettings();
    }

    const newSettings: ISettings = {
      ...settings,
      recents,
    };
    await utils.ipc.settings[''].notify(newSettings);

    return newSettings;
  };

  const mockEntryAndExitLocations = async (
    {
      entryLocation,
      exitLocation,
    }: {
      entryLocation: Constraint<RelayLocation>;
      exitLocation: Constraint<RelayLocation>;
    },
    settings?: ISettings,
  ) => {
    if (!settings) {
      settings = getDefaultSettings();
    }

    if ('normal' in settings.relaySettings) {
      settings.relaySettings.normal.location = exitLocation;
      settings.relaySettings.normal.wireguardConstraints.entryLocation = entryLocation;
    }
    await utils.ipc.settings[''].notify(settings);

    return settings;
  };

  const mockConnectTunnel = async (location: ILocation) => {
    await utils.ipc.tunnel[''].notify({
      state: 'connected',
      details: {
        endpoint: {
          entryEndpoint: {
            address: '',
            transportProtocol: 'tcp',
          },
          address: '',
          daita: false,
          protocol: 'tcp',
          quantumResistant: false,
        },
        location,
      },
    });
  };

  return {
    areAllCheckboxesChecked,
    expandLocatedRelays,
    toSelectionPaths,
    getRelaysByHostnames,
    getRelaysByOwner,
    getRelaysByProvider,
    getRelaysByObfuscation,
    getProvidersFromRelayList,
    resetOwnership,
    resetProviders,
    resetToFilterView,
    resetToSelectLocationView,
    updateMockRelayFilter,
    updateMockEntryRelayFilter,
    updateMockExitRelayFilter,
    mockSettings,
    mockEntryLocation,
    mockExitLocation,
    mockCustomLists,
    mockRecents,
    mockEntryAndExitLocations,
    mockConnectTunnel,
  };
};

export type SelectLocationHelpers = ReturnType<typeof createHelpers>;
