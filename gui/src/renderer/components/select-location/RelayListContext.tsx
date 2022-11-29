import React, { useCallback, useContext, useEffect, useMemo, useState } from 'react';

import { compareRelayLocation, RelayLocation } from '../../../shared/daemon-rpc-types';
import {
  EndpointType,
  filterLocations,
  filterLocationsByEndPointType,
  getLocationsExpandedBySearch,
  searchForLocations,
} from '../../lib/filter-locations';
import { useNormalBridgeSettings, useNormalRelaySettings } from '../../lib/utilityHooks';
import { IRelayLocationRedux } from '../../redux/settings/reducers';
import { useSelector } from '../../redux/store';
import { useScrollPositionContext } from './ScrollPositionContext';
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
  LocationSelectionType,
  LocationType,
} from './select-location-types';
import { useSelectLocationContext } from './SelectLocationContainer';

// Context containing the relay list and related data and callbacks
interface RelayListContext {
  relayList: LocationList<never>;
  expandLocation: (location: RelayLocation) => void;
  collapseLocation: (location: RelayLocation) => void;
  onBeforeExpand: (
    locationRect: DOMRect,
    expandedContentHeight: number,
    invokedByUser: boolean,
  ) => void;
  expandSearchResults: (searchTerm: string) => void;
}

type ExpandedLocations = Partial<Record<LocationType, Array<RelayLocation>>>;

const relayListContext = React.createContext<RelayListContext | undefined>(undefined);

export function useRelayListContext() {
  return useContext(relayListContext)!;
}

interface RelayListContextProviderProps {
  children: React.ReactNode;
}

export function RelayListContextProvider(props: RelayListContextProviderProps) {
  const { locationType, searchTerm } = useSelectLocationContext();
  const fullRelayList = useSelector((state) => state.settings.relayLocations);
  const relaySettings = useNormalRelaySettings();

  // Filters the relays to only keep the ones of the desired endpoint type, e.g. "wireguard",
  // "openvpn" or "bridge"
  const relayListForEndpointType = useMemo(() => {
    const endpointType =
      locationType === LocationType.entry ? EndpointType.entry : EndpointType.exit;
    return filterLocationsByEndPointType(fullRelayList, endpointType, relaySettings);
  }, [fullRelayList, locationType, relaySettings?.tunnelProtocol]);

  // Filters the relays to only keep the relays matching the currently selected filters, e.g.
  // ownership and providers
  const relayListForFilters = useMemo(() => {
    return filterLocations(
      relayListForEndpointType,
      relaySettings?.ownership,
      relaySettings?.providers,
    );
  }, [relaySettings?.ownership, relaySettings?.providers, relayListForEndpointType]);

  // Filters the relays based on the provided search term
  const relayListForSearch = useMemo(() => {
    return searchForLocations(relayListForFilters, searchTerm);
  }, [relayListForFilters, searchTerm]);

  const {
    expandedLocations,
    expandLocation,
    collapseLocation,
    onBeforeExpand,
    expandSearchResults,
  } = useExpandedLocations(relayListForFilters);

  // Prepares all relays and combines the data needed for rendering them
  const relayList = useRelayList(relayListForSearch, expandedLocations);

  const contextValue = useMemo(
    () => ({
      relayList,
      expandLocation,
      collapseLocation,
      onBeforeExpand,
      expandSearchResults,
    }),
    [relayList, expandLocation, collapseLocation, onBeforeExpand, expandSearchResults],
  );

  return (
    <relayListContext.Provider value={contextValue}>{props.children}</relayListContext.Provider>
  );
}

// Return the final filtered and formatted relay list. This should be the only place in the app
// where processing of the relay list is performed.
function useRelayList(
  relayList: Array<IRelayLocationRedux>,
  expandedLocations?: Array<RelayLocation>,
): LocationList<never> {
  const locale = useSelector((state) => state.userInterface.locale);
  const selectedLocation = useSelectedLocation();
  const disabledLocation = useDisabledLocation();

  return useMemo(() => {
    return relayList
      .map((country) => {
        const countryLocation = { country: country.code };
        const countryDisabled = isCountryDisabled(country, country.code, disabledLocation);

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

// Return all RelayLocations that should be expanded
function useExpandedLocations(filteredLocations: Array<IRelayLocationRedux>) {
  const { locationType, searchTerm } = useSelectLocationContext();
  const { spacePreAllocationViewRef, scrollIntoView } = useScrollPositionContext();
  const relaySettings = useNormalRelaySettings();
  const bridgeSettings = useNormalBridgeSettings();

  // Keeps the state of which locations are expanded for which locationType. This is used to restore
  // the state when switching back and forth between entry and exit.
  const [expandedLocationsMap, setExpandedLocations] = useState<ExpandedLocations>(() =>
    defaultExpandedLocations(relaySettings, bridgeSettings),
  );

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

  // Called before expansion to make room for expansion and to scroll to fit the element
  const onBeforeExpand = useCallback(
    (locationRect: DOMRect, expandedContentHeight: number, invokedByUser: boolean) => {
      if (invokedByUser) {
        locationRect.height += expandedContentHeight;
        spacePreAllocationViewRef.current?.allocate(expandedContentHeight);
        scrollIntoView(locationRect);
      }
    },
    [],
  );

  // Expand search results when searching
  const expandSearchResults = useCallback(
    (searchTerm: string) => {
      if (searchTerm === '') {
        setExpandedLocations(defaultExpandedLocations(relaySettings, bridgeSettings));
      } else {
        setExpandedLocations((expandedLocations) => ({
          ...expandedLocations,
          [locationType]: getLocationsExpandedBySearch(filteredLocations, searchTerm),
        }));
      }
    },
    [relaySettings, bridgeSettings, locationType, filteredLocations],
  );

  // Expand locations when filters are changed
  useEffect(() => {
    if (searchTerm !== '') {
      setExpandedLocations((expandedLocations) => ({
        ...expandedLocations,
        [locationType]: getLocationsExpandedBySearch(filteredLocations, searchTerm),
      }));
    }
  }, [filteredLocations]);

  return {
    expandedLocations: expandedLocationsMap[locationType],
    expandLocation,
    collapseLocation,
    onBeforeExpand,
    expandSearchResults,
  };
}

// Returns the location (if any) that should be disabled. This is currently used for disabling the
// entry location when selecting exit location etc.
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
  }, [
    locationType,
    relaySettings?.tunnelProtocol,
    relaySettings?.wireguard.useMultihop,
    relaySettings?.wireguard.entryLocation,
    relaySettings?.location,
  ]);
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
  }, [
    locationType,
    relaySettings?.location,
    relaySettings?.tunnelProtocol,
    relaySettings?.wireguard.entryLocation,
    bridgeSettings?.location,
  ]);
}
