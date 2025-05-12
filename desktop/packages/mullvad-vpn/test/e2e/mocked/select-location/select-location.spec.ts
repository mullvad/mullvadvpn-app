import { expect, test } from '@playwright/test';
import { Page } from 'playwright';

import { getDefaultSettings } from '../../../../src/main/default-settings';
import { colors } from '../../../../src/renderer/lib/foundations';
import { RoutePath } from '../../../../src/renderer/lib/routes';
import {
  IRelayList,
  IRelayListCity,
  IRelayListCountry,
  IRelayListHostname,
  IRelayListWithEndpointData,
  ISettings,
  IWireguardEndpointData,
  Ownership,
} from '../../../../src/shared/daemon-rpc-types';
import { MockedTestUtils, startMockedApp } from '../mocked-utils';
import { createSelectors } from './helpers';

const relayList: IRelayList = {
  countries: [
    {
      name: 'Sweden',
      code: 'se',
      cities: [
        {
          name: 'Gothenburg',
          code: 'got',
          latitude: 58,
          longitude: 12,
          relays: [
            {
              hostname: 'se-got-wg-101',
              provider: 'mullvad',
              ipv4AddrIn: '10.0.0.1',
              includeInCountry: true,
              active: true,
              weight: 0,
              owned: true,
              endpointType: 'wireguard',
              daita: true,
            },
            {
              hostname: 'se-got-wg-102',
              provider: 'mullvad',
              ipv4AddrIn: '10.0.0.2',
              includeInCountry: true,
              active: true,
              weight: 0,
              owned: true,
              endpointType: 'wireguard',
              daita: true,
            },
            {
              hostname: 'se-got-wg-103',
              provider: 'another-provider',
              ipv4AddrIn: '10.0.0.3',
              includeInCountry: true,
              active: true,
              weight: 0,
              owned: false,
              endpointType: 'wireguard',
              daita: true,
            },
          ],
        },
      ],
    },
  ],
};

const wireguardEndpointData: IWireguardEndpointData = {
  portRanges: [],
  udp2tcpPorts: [],
};

let page: Page;
let util: MockedTestUtils;
let selectors: ReturnType<typeof createSelectors>;

test.beforeAll(async () => {
  ({ page, util } = await startMockedApp());
  selectors = createSelectors(page);
  await util.waitForRoute(RoutePath.main);
  await setMultihop();
  await page.getByLabel('Select location').click();
  await util.waitForRoute(RoutePath.selectLocation);
});

test.afterAll(async () => {
  await page.close();
});

async function setMultihop() {
  const settings = getDefaultSettings();
  if ('normal' in settings.relaySettings) {
    settings.relaySettings.normal.wireguardConstraints.useMultihop = true;
  }

  await util.sendMockIpcResponse<ISettings>({
    channel: 'settings-',
    response: settings,
  });

  await util.sendMockIpcResponse<IRelayListWithEndpointData>({
    channel: 'relays-',
    response: { relayList, wireguardEndpointData },
  });
}

test('App should show entry selection', async () => {
  const entryTab = page.getByText('Entry');
  await entryTab.click();
  await expect(entryTab).toHaveCSS('background-color', colors['--color-green']);

  const sweden = page.getByText('Sweden');
  await expect(sweden).toBeVisible();
});

test('App should show exit selection', async () => {
  const exitTab = page.getByText('Exit');
  await exitTab.click();
  await expect(exitTab).toHaveCSS('background-color', colors['--color-green']);

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
  await expect(entryTab).toHaveCSS('background-color', colors['--color-green']);

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
  await expect(entryTab).toHaveCSS('background-color', colors['--color-green']);

  const sweden = page.getByText('Sweden');
  await expect(sweden).toBeVisible();
});

type LocatedRelay = {
  country: IRelayListCountry;
  city: IRelayListCity;
  relay: IRelayListHostname;
};

test.describe('Filter', () => {
  const expandLocatedRelays = async (locatedRelays: LocatedRelay[]) => {
    for (const locatedRelay of locatedRelays) {
      const expandCountry = page.getByLabel(`Expand ${locatedRelay.country.name}`);
      if (await expandCountry.count()) {
        await expandCountry.click();
      }
      const expandCity = page.getByLabel(`Expand ${locatedRelay.city.name}`);
      if (await expandCity.count()) {
        await expandCity.click();
      }
    }
  };

  const resetView = async () => {
    const currentRoute = await util.currentRoute();
    // Reset view by navigating back to select location
    // This can be improved once we have a better way to select the accordions
    if (currentRoute === RoutePath.filter) {
      await selectors.backButton().click();
      await util.waitForRoute(RoutePath.selectLocation);
    }
    // Ensure we start on the filter view
    await selectors.filterButton().click();
    await util.waitForRoute(RoutePath.filter);
  };

  const resetProviders = async () => {
    // Ensure all providers are selected
    await selectors.accordion('Providers').click();
    const checkboxes = page.getByRole('checkbox');
    if (
      await checkboxes.evaluateAll((checkboxes) =>
        checkboxes.some((checkbox) => checkbox.getAttribute('aria-checked') === 'false'),
      )
    ) {
      await selectors.allProvidersCheckbox().click();
    }
    await selectors.accordion('Providers').click();
  };

  const resetOwnership = async () => {
    // Ensure any owner is selected
    const ownershipAccordion = selectors.accordion('Ownership');
    await ownershipAccordion.click();
    await selectors.option('Any').click();
    await ownershipAccordion.click();
  };

  const mockRelayFilter = async ({
    ownership,
    provider,
  }: {
    ownership?: Ownership;
    provider?: string[];
  }) => {
    const settings = getDefaultSettings();
    if ('normal' in settings.relaySettings) {
      if (ownership) {
        settings.relaySettings.normal.ownership = ownership;
      }
      if (provider) {
        settings.relaySettings.normal.providers = provider;
      }
    }
    await util.sendMockIpcResponse<IRelayListWithEndpointData>({
      channel: 'relays-',
      response: { relayList, wireguardEndpointData },
    });
    await util.mockIpcHandle({
      channel: 'settings-setRelaySettings',
      response: {},
    });
    await util.sendMockIpcResponse({
      channel: 'settings-',
      response: settings,
    });
  };

  test.beforeEach(async () => {
    await resetView();
    await resetProviders();
    await resetOwnership();
  });

  test.describe('Filter by provider', () => {
    const locateRelayByProvider = (provider?: string): LocatedRelay[] => {
      const results: LocatedRelay[] = [];

      for (const country of relayList.countries) {
        for (const city of country.cities) {
          for (const relay of city.relays) {
            if (!provider || relay.provider === provider) {
              results.push({
                country: country,
                city: city,
                relay: relay,
              });
            }
          }
        }
      }

      return results;
    };

    const expectAllCheckboxesChecked = async (checked: boolean) => {
      const checkboxes = page.getByRole('checkbox');
      expect(
        await checkboxes.evaluateAll(
          (elements, checked) =>
            elements.every(
              (element) => element.getAttribute('aria-checked') === checked.toString(),
            ),
          checked,
        ),
      ).toBeTruthy();
    };

    test.afterEach(async () => {});

    test('Should deselect all providers when clicking all providers checkbox', async () => {
      await selectors.accordion('Providers').click();
      const allProvidersCheckbox = selectors.allProvidersCheckbox();
      await allProvidersCheckbox.click();
      await expectAllCheckboxesChecked(false);

      await allProvidersCheckbox.click();
      await expectAllCheckboxesChecked(true);
    });

    test('Should apply apply filter when clicking selecting provider', async () => {
      await selectors.accordion('Providers').click();

      // Deselect all providers
      await selectors.allProvidersCheckbox().click();
      await expectAllCheckboxesChecked(false);

      // Select one provider
      const provider = relayList.countries[0].cities[0].relays[0].provider;
      const providerCheckbox = page.getByLabel(provider);
      await providerCheckbox.click();

      await mockRelayFilter({ provider: [provider] });

      await selectors.applyButton().click();
      await util.waitForRoute(RoutePath.selectLocation);
      const providerFilterChip = selectors.providerFilterChip(1);
      await expect(providerFilterChip).toBeVisible();

      const locatedRelays = locateRelayByProvider(provider);
      const relays = locatedRelays.map((locatedRelay) => locatedRelay.relay);
      const relayNames = relays.map((relay) => relay.hostname);

      // Expand all accordions
      await expandLocatedRelays(locatedRelays);

      const buttons = page.getByRole('button', {
        name: new RegExp(relayNames.join('|')),
      });

      // Expect all filtered relays to have a button
      await expect(buttons).toHaveCount(relays.length);

      // Clear filter
      await providerFilterChip.click();

      // Get all relays and expand accordions
      const allLocatedRelays = locateRelayByProvider();
      await expandLocatedRelays(allLocatedRelays);

      // Should not have same length as all relays
      await expect(buttons).not.toHaveCount(allLocatedRelays.length);
    });
  });

  test.describe('Filter by ownership', () => {
    const locateRelaysByOwner = (owned?: boolean): LocatedRelay[] => {
      const results: LocatedRelay[] = [];

      for (const country of relayList.countries) {
        for (const city of country.cities) {
          for (const relay of city.relays) {
            if (owned === undefined || relay.owned === owned) {
              results.push({
                country: country,
                city: city,
                relay: relay,
              });
            }
          }
        }
      }

      return results;
    };

    test('Should apply apply filter when clicking selecting ownership', async () => {
      await selectors.accordion('Ownership').click();

      // Select rented only
      await selectors.option('Rented only').click();
      await mockRelayFilter({ ownership: Ownership.rented });
      await selectors.applyButton().click();
      await util.waitForRoute(RoutePath.selectLocation);

      const providerFilterChip = selectors.ownerFilterChip(false);
      await expect(providerFilterChip).toBeVisible();

      const locatedRelays = locateRelaysByOwner(false);
      const relays = locatedRelays.map((locatedRelay) => locatedRelay.relay);
      const relayNames = relays.map((relay) => relay.hostname);

      // Expand all accordions
      await expandLocatedRelays(locatedRelays);

      const buttons = page.getByRole('button', {
        name: new RegExp(relayNames.join('|')),
      });

      // Expect all filtered relays to have a button
      await expect(buttons).toHaveCount(relays.length);

      // Clear filter
      await providerFilterChip.click();

      // Get all relays and expand accordions
      const allLocatedRelays = locateRelaysByOwner();
      await expandLocatedRelays(allLocatedRelays);

      // Should not have same length as all relays
      await expect(buttons).not.toHaveCount(allLocatedRelays.length);
    });
  });
});
