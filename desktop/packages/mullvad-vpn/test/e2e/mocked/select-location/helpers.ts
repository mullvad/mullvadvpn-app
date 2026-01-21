import { Page } from 'playwright';

import { getDefaultSettings } from '../../../../src/main/default-settings';
import {
  IRelayList,
  IRelayListCity,
  IRelayListCountry,
  IRelayListHostname,
  ISettings,
  Ownership,
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
      await routes.selectLocation.toggleAccordion(locatedRelay.country.name);
      await routes.selectLocation.toggleAccordion(locatedRelay.city.name);
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

  const resetView = async () => {
    const currentRoute = await utils.getCurrentRoute();
    if (currentRoute === RoutePath.selectLocation) {
      await routes.selectLocation.gotoFilter();
    }
  };

  const updateMockRelayFilter = async ({
    ownership,
    providers,
  }: {
    ownership?: Ownership;
    providers?: string[];
  }) => {
    const settings = getDefaultSettings();
    if ('normal' in settings.relaySettings) {
      if (ownership) {
        settings.relaySettings.normal.ownership = ownership;
      }
      if (providers) {
        settings.relaySettings.normal.providers = providers;
      }
    }
    await utils.ipc.settings[''].notify(settings);
  };

  const updateMockSettings = async (
    {
      daita,
      directOnly,
      multihop,
    }: {
      multihop?: boolean;
      daita?: boolean;
      directOnly?: boolean;
    },
    settings?: ISettings,
  ) => {
    if (!settings) {
      settings = getDefaultSettings();
    }
    if ('normal' in settings.relaySettings && settings.tunnelOptions.daita) {
      if (multihop !== undefined)
        settings.relaySettings.normal.wireguardConstraints.useMultihop = multihop;
      if (daita !== undefined) settings.tunnelOptions.daita.enabled = daita;
      if (directOnly !== undefined) settings.tunnelOptions.daita.directOnly = directOnly;
    }

    await utils.ipc.settings[''].notify(settings);

    return settings;
  };

  const updateEntryLocation = async (relay: RelaySelectionPath, settings?: ISettings) => {
    if (!settings) {
      settings = getDefaultSettings();
    }
    if ('normal' in settings.relaySettings && settings.tunnelOptions.daita) {
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

  return {
    areAllCheckboxesChecked,
    expandLocatedRelays,
    toSelectionPaths,
    getRelaysByHostnames,
    getRelaysByOwner,
    getRelaysByProvider,
    getRelaysByObfuscation,
    resetOwnership,
    resetProviders,
    resetView,
    updateMockRelayFilter,
    updateMockSettings,
    updateEntryLocation,
  };
};

export type SelectLocationHelpers = ReturnType<typeof createHelpers>;
