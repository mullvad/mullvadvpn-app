import { Page } from 'playwright';

import { getDefaultSettings } from '../../../../src/main/default-settings';
import {
  IRelayList,
  IRelayListCity,
  IRelayListCountry,
  IRelayListHostname,
  Ownership,
} from '../../../../src/shared/daemon-rpc-types';
import { RoutePath } from '../../../../src/shared/routes';
import { RoutesObjectModel } from '../../route-object-models';
import { MockedTestUtils } from '../mocked-utils';

export type LocatedRelay = {
  country: IRelayListCountry;
  city: IRelayListCity;
  relay: IRelayListHostname;
};

export const createHelpers = (page: Page, routes: RoutesObjectModel, utils: MockedTestUtils) => {
  const areAllCheckboxesChecked = async () => {
    const checkboxes = page.getByRole('checkbox');
    return checkboxes.evaluateAll((checkboxes) =>
      checkboxes.every((checkbox) => checkbox.getAttribute('aria-checked') === 'true'),
    );
  };

  const expandLocatedRelays = async (locatedRelays: LocatedRelay[]) => {
    for (const locatedRelay of locatedRelays) {
      await routes.selectLocation.toggleAccordion(locatedRelay.country.name);
      await routes.selectLocation.toggleAccordion(locatedRelay.city.name);
    }
  };

  const locateRelaysByProvider = (relayList: IRelayList, provider?: string): LocatedRelay[] =>
    relayList.countries.flatMap((country) =>
      country.cities.flatMap((city) =>
        city.relays
          .filter((relay) => !provider || relay.provider === provider)
          .map((relay) => ({ country, city, relay })),
      ),
    );

  const locateRelaysByOwner = (relayList: IRelayList, owned?: boolean): LocatedRelay[] =>
    relayList.countries.flatMap((country) =>
      country.cities.flatMap((city) =>
        city.relays
          .filter((relay) => relay.owned === owned)
          .map((relay) => ({ country, city, relay })),
      ),
    );

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
    const currentRoute = await utils.currentRoute();
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
    await utils.mockIpcHandle({
      channel: 'settings-setRelaySettings',
      response: {},
    });
    await utils.sendMockIpcResponse({
      channel: 'settings-',
      response: settings,
    });
  };

  return {
    areAllCheckboxesChecked,
    expandLocatedRelays,
    locateRelaysByProvider,
    locateRelaysByOwner,
    resetOwnership,
    resetProviders,
    resetView,
    updateMockRelayFilter,
  };
};

export type SelectLocationHelpers = ReturnType<typeof createHelpers>;
