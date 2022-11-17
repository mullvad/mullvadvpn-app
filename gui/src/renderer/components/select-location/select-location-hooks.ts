import { useCallback, useMemo } from 'react';

import BridgeSettingsBuilder from '../../../shared/bridge-settings-builder';
import {
  compareRelayLocation,
  RelayLocation,
  RelaySettingsUpdate,
} from '../../../shared/daemon-rpc-types';
import log from '../../../shared/logging';
import RelaySettingsBuilder from '../../../shared/relay-settings-builder';
import { useAppContext } from '../../context';
import { createWireguardRelayUpdater } from '../../lib/constraint-updater';
import {
  EndpointType,
  filterLocations,
  filterLocationsByEndPointType,
  getLocationsExpandedBySearch,
  searchForLocations,
} from '../../lib/filter-locations';
import { useHistory } from '../../lib/history';
import {
  useNormalBridgeSettings,
  useNormalRelaySettings,
  useSharedMemo,
} from '../../lib/utilityHooks';
import { IRelayLocationRedux } from '../../redux/settings/reducers';
import { useSelector } from '../../redux/store';
import {
  defaultExpandedLocations,
  formatRowName,
  isCityDisabled,
  isCountryDisabled,
  isExpanded,
  isRelayDisabled,
  isSelected,
} from './select-location-helpers';
import {
  DisabledReason,
  LocationList,
  LocationSelection,
  LocationSelectionType,
  LocationType,
  SpecialBridgeLocationType,
} from './select-location-types';
import { useSelectLocationContext } from './SelectLocationContainer';

// Return all locations that matches both the set filters and the search term.
function useFilteredRelays(): Array<IRelayLocationRedux> {
  const { locationType, searchTerm } = useSelectLocationContext();
  const relayList = useSelector((state) => state.settings.relayLocations);
  const relaySettings = useNormalRelaySettings();

  const relayListForEndpointType = useSharedMemo(
    'relay-list-endpoint-type',
    () => {
      const endpointType =
        locationType === LocationType.entry ? EndpointType.entry : EndpointType.exit;
      return filterLocationsByEndPointType(relayList, endpointType, relaySettings);
    },
    [relayList, locationType, relaySettings?.tunnelProtocol],
  );

  const relayListForFilters = useSharedMemo(
    'relay-list-filters',
    () => {
      return filterLocations(
        relayListForEndpointType,
        relaySettings?.ownership,
        relaySettings?.providers,
      );
    },
    [relaySettings?.ownership, relaySettings?.providers, relayListForEndpointType],
  );

  const filteredRelayList = useSharedMemo(
    'relay-list-search',
    () => {
      return searchForLocations(relayListForFilters, searchTerm);
    },
    [relayListForFilters, searchTerm],
  );

  return filteredRelayList;
}

// Return all RelayLocations that should be expanded
export function useExpandedLocations() {
  const relaySettings = useNormalRelaySettings();
  const bridgeSettings = useNormalBridgeSettings();
  const {
    locationType,
    expandedLocations,
    setExpandedLocations,
    searchTerm,
  } = useSelectLocationContext();
  const relayList = useFilteredRelays();

  const expandLocation = useCallback(
    (location: RelayLocation) => {
      setExpandedLocations((expandedLocations) => ({
        ...expandedLocations,
        [locationType]: [...(expandedLocations[locationType] ?? []), location],
      }));
    },
    [locationType],
  );

  const collapseLocation = useCallback(
    (location: RelayLocation) => {
      setExpandedLocations((expandedLocations) => ({
        ...expandedLocations,
        [locationType]: expandedLocations[locationType]!.filter(
          (item) => !compareRelayLocation(location, item),
        ),
      }));
    },
    [locationType],
  );

  const updateExpandedLocations = useCallback(() => {
    if (searchTerm === '') {
      setExpandedLocations(defaultExpandedLocations(relaySettings, bridgeSettings));
    } else {
      setExpandedLocations((expandedLocations) => ({
        ...expandedLocations,
        [locationType]: getLocationsExpandedBySearch(relayList, searchTerm),
      }));
    }
  }, [relayList, searchTerm, relaySettings?.ownership, relaySettings?.providers]);

  return {
    expandedLocations: expandedLocations[locationType],
    expandLocation,
    collapseLocation,
    updateExpandedLocations,
  };
}

// Return the final filtered and formatted relay list. This should be the only place in the app
// where processing of the relay list is performed.
export function useRelayList(): LocationList<never> {
  const locale = useSelector((state) => state.userInterface.locale);
  const { expandedLocations } = useExpandedLocations();
  const relayList = useFilteredRelays();
  const selectedLocation = useSelectedLocation();
  const disabledLocation = useDisabledLocation();

  return useSharedMemo('relay-list-formatted', () => {
    return relayList
      .map((country) => {
        const countryLocation = { country: country.code };
        const countryDisabled = isCountryDisabled(country, countryLocation.country, disabledLocation);

        return {
          ...country,
          type: LocationSelectionType.relay as const,
          label: formatRowName(country.name, countryLocation, countryDisabled),
          location: countryLocation,
          active: countryDisabled !== DisabledReason.inactive,
          disabled: countryDisabled !== undefined,
          expanded: isExpanded(countryLocation, expandedLocations),
          selected: isSelected(countryLocation, selectedLocation),
          cities: country.cities
            .map((city) => {
              const cityLocation: RelayLocation = { city: [country.code, city.code] };
              const cityDisabled =
                countryDisabled ?? isCityDisabled(city, cityLocation.city, disabledLocation);

              return {
                ...city,
                label: formatRowName(city.name, cityLocation, cityDisabled),
                location: cityLocation,
                active: cityDisabled !== DisabledReason.inactive,
                disabled: cityDisabled !== undefined,
                expanded: isExpanded(cityLocation, expandedLocations),
                selected: isSelected(cityLocation, selectedLocation),
                relays: city.relays
                  .map((relay) => {
                    const relayLocation: RelayLocation = {
                      hostname: [country.code, city.code, relay.hostname],
                    };
                    const relayDisabled =
                      countryDisabled ??
                      cityDisabled ??
                      isRelayDisabled(relay, relayLocation.hostname, disabledLocation);

                    return {
                      ...relay,
                      label: formatRowName(relay.hostname, relayLocation, relayDisabled),
                      location: relayLocation,
                      disabled: relayDisabled !== undefined,
                      selected: isSelected(relayLocation, selectedLocation),
                    };
                  })
                  .sort((a, b) => a.hostname.localeCompare(b.hostname, locale, { numeric: true })),
              };
            })
            .sort((a, b) => a.label.localeCompare(b.label, locale)),
        };
      })
      .sort((a, b) => a.label.localeCompare(b.label, locale));
  }, [locale, expandedLocations, relayList, selectedLocation, disabledLocation]);
}

function useDisabledLocation() {
  const { locationType } = useSelectLocationContext();
  const relaySettings = useNormalRelaySettings();

  return useMemo(() => {
    if (relaySettings?.tunnelProtocol !== 'openvpn' && relaySettings?.wireguard.useMultihop) {
      if (locationType === LocationType.exit && relaySettings?.wireguard.entryLocation !== 'any') {
        return {
          location: relaySettings?.wireguard.entryLocation,
          reason: DisabledReason.entry,
        };
      } else if (locationType === LocationType.entry && relaySettings?.location !== 'any') {
        return { location: relaySettings?.location, reason: DisabledReason.exit };
      }
    }

    return undefined;
  }, [locationType, relaySettings?.tunnelProtocol, relaySettings?.wireguard.useMultihop, relaySettings?.wireguard.entryLocation, relaySettings?.location]);
}

// Returns the selected location for the current tunnel protocol and location type
function useSelectedLocation() {
  const { locationType } = useSelectLocationContext();
  const relaySettings = useNormalRelaySettings();
  const bridgeSettings = useNormalBridgeSettings();

  return useMemo(() => {
    if (locationType === LocationType.exit) {
      return relaySettings?.location === 'any' ? undefined : relaySettings?.location;
    } else if (relaySettings?.tunnelProtocol !== 'openvpn') {
      return relaySettings?.wireguard.entryLocation === 'any'
        ? undefined
        : relaySettings?.wireguard.entryLocation;
    } else {
      return bridgeSettings?.location;
    }
  }, [locationType, relaySettings?.location, relaySettings?.tunnelProtocol, relaySettings?.wireguard.entryLocation, bridgeSettings?.location]);
}

export function useOnSelectLocation() {
  const history = useHistory();
  const { updateRelaySettings } = useAppContext();
  const { locationType } = useSelectLocationContext();
  const baseRelaySettings = useSelector((state) => state.settings.relaySettings);

  const onSelectLocation = useCallback(
    async (relayUpdate: RelaySettingsUpdate) => {
      // dismiss the view first
      history.dismiss();
      try {
        await updateRelaySettings(relayUpdate);
      } catch (e) {
        const error = e as Error;
        log.error(`Failed to select the exit location: ${error.message}`);
      }
    },
    [history],
  );

  const onSelectExitLocation = useCallback(
    async (relayLocation: LocationSelection<never>) => {
      const relayUpdate = RelaySettingsBuilder.normal()
        .location.fromRaw(relayLocation.value)
        .build();
      await onSelectLocation(relayUpdate);
    },
    [onSelectLocation],
  );
  const onSelectEntryLocation = useCallback(
    async (entryLocation: LocationSelection<never>) => {
      const relayUpdate = createWireguardRelayUpdater(baseRelaySettings)
        .tunnel.wireguard((wireguard) => wireguard.entryLocation.exact(entryLocation.value))
        .build();
      await onSelectLocation(relayUpdate);
    },
    [onSelectLocation],
  );

  return locationType === LocationType.exit ? onSelectExitLocation : onSelectEntryLocation;
}

export function useOnSelectBridgeLocation() {
  const history = useHistory();
  const { updateBridgeSettings } = useAppContext();

  return useCallback(
    async (location: LocationSelection<SpecialBridgeLocationType>) => {
      // dismiss the view first
      history.dismiss();

      let bridgeUpdate;
      if (location.type === LocationSelectionType.relay) {
        bridgeUpdate = new BridgeSettingsBuilder().location.fromRaw(location.value).build();
      } else if (
        location.type === LocationSelectionType.special &&
        location.value === SpecialBridgeLocationType.closestToExit
      ) {
        bridgeUpdate = new BridgeSettingsBuilder().location.any().build();
      }

      if (bridgeUpdate) {
        try {
          await updateBridgeSettings(bridgeUpdate);
        } catch (e) {
          const error = e as Error;
          log.error(`Failed to select the bridge location: ${error.message}`);
        }
      }
    },
    [history, updateBridgeSettings],
  );
}
