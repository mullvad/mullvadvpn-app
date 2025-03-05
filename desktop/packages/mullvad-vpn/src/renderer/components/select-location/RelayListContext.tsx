import React, { useCallback, useContext, useEffect, useMemo, useState } from 'react';

import { compareRelayLocation, RelayLocation } from '../../../shared/daemon-rpc-types';
import {
  EndpointType,
  filterLocations,
  filterLocationsByDaita,
  filterLocationsByEndPointType,
  getLocationsExpandedBySearch,
  searchForLocations,
} from '../../lib/filter-locations';
import {
  useNormalBridgeSettings,
  useNormalRelaySettings,
  useTunnelProtocol,
} from '../../lib/relay-settings-hooks';
import { useEffectEvent } from '../../lib/utility-hooks';
import { IRelayLocationCountryRedux } from '../../redux/settings/reducers';
import { useSelector } from '../../redux/store';
import { useCustomListsRelayList } from './custom-list-helpers';
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
  CustomListSpecification,
  DisabledReason,
  GeographicalRelayList,
  LocationType,
  RelayLocationCountryWithVisibility,
} from './select-location-types';
import { useSelectLocationContext } from './SelectLocationContainer';

// Context containing the relay list and related data and callbacks
interface RelayListContext {
  relayList: GeographicalRelayList;
  customLists: Array<CustomListSpecification>;
  expandedLocations?: Array<RelayLocation>;
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

export const relayListContext = React.createContext<RelayListContext | undefined>(undefined);

export function useRelayListContext() {
  return useContext(relayListContext)!;
}

interface RelayListContextProviderProps {
  children: React.ReactNode;
}

export function RelayListContextProvider(props: RelayListContextProviderProps) {
  const { locationType, searchTerm } = useSelectLocationContext();
  const daita = useSelector((state) => state.settings.wireguard.daita?.enabled ?? false);
  const directOnly = useSelector((state) => state.settings.wireguard.daita?.directOnly ?? false);

  const fullRelayList = useSelector((state) => state.settings.relayLocations);
  const relaySettings = useNormalRelaySettings();
  const tunnelProtocol = useTunnelProtocol();

  // Filters the relays to only keep the ones of the desired endpoint type, e.g. "wireguard",
  // "openvpn" or "bridge"
  const relayListForEndpointType = useMemo(() => {
    const endpointType =
      locationType === LocationType.entry ? EndpointType.entry : EndpointType.exit;
    return filterLocationsByEndPointType(
      fullRelayList,
      endpointType,
      tunnelProtocol,
      relaySettings,
    );
  }, [fullRelayList, locationType, relaySettings, tunnelProtocol]);

  const relayListForDaita = useMemo(() => {
    return filterLocationsByDaita(
      relayListForEndpointType,
      daita,
      directOnly,
      locationType,
      relaySettings?.tunnelProtocol,
      relaySettings?.wireguard.useMultihop ?? false,
    );
  }, [
    daita,
    directOnly,
    locationType,
    relayListForEndpointType,
    relaySettings?.tunnelProtocol,
    relaySettings?.wireguard.useMultihop,
  ]);

  // Filters the relays to only keep the relays matching the currently selected filters, e.g.
  // ownership and providers
  const relayListForFilters = useMemo(() => {
    return filterLocations(relayListForDaita, relaySettings?.ownership, relaySettings?.providers);
  }, [relaySettings?.ownership, relaySettings?.providers, relayListForDaita]);

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

  const customLists = useCustomListsRelayList(relayList, expandedLocations);

  const contextValue = useMemo(
    () => ({
      relayList,
      customLists,
      expandedLocations,
      expandLocation,
      collapseLocation,
      onBeforeExpand,
      expandSearchResults,
    }),
    [
      relayList,
      customLists,
      expandedLocations,
      expandLocation,
      collapseLocation,
      onBeforeExpand,
      expandSearchResults,
    ],
  );

  return (
    <relayListContext.Provider value={contextValue}>{props.children}</relayListContext.Provider>
  );
}

// Return the final filtered and formatted relay list. This should be the only place in the app
// where processing of the relay list is performed.
function useRelayList(
  relayList: Array<RelayLocationCountryWithVisibility>,
  expandedLocations?: Array<RelayLocation>,
): GeographicalRelayList {
  const locale = useSelector((state) => state.userInterface.locale);
  const selectedLocation = useSelectedLocation();
  const disabledLocation = useDisabledLocation();

  const preventDueToCustomBridgeSelected = usePreventDueToCustomBridgeSelected();

  const isLocationSelected = useCallback(
    (location: RelayLocation) => {
      return preventDueToCustomBridgeSelected ? false : isSelected(location, selectedLocation);
    },
    [preventDueToCustomBridgeSelected, selectedLocation],
  );

  return useMemo(() => {
    return relayList
      .map((country) => {
        const countryLocation = { country: country.code };
        const countryDisabledReason = isCountryDisabled(country, countryLocation, disabledLocation);

        return {
          ...country,
          label: formatRowName(country.name, countryLocation, countryDisabledReason),
          location: countryLocation,
          active: countryDisabledReason !== DisabledReason.inactive,
          disabled: countryDisabledReason !== undefined,
          disabledReason: countryDisabledReason,
          expanded: isExpanded(countryLocation, expandedLocations),
          selected: isLocationSelected(countryLocation),
          cities: country.cities
            .map((city) => {
              const cityLocation: RelayLocation = { country: country.code, city: city.code };
              const cityDisabledReason =
                countryDisabledReason ?? isCityDisabled(city, cityLocation, disabledLocation);

              return {
                ...city,
                label: formatRowName(city.name, cityLocation, cityDisabledReason),
                location: cityLocation,
                active: cityDisabledReason !== DisabledReason.inactive,
                disabled: cityDisabledReason !== undefined,
                disabledReason: cityDisabledReason,
                expanded: isExpanded(cityLocation, expandedLocations),
                selected: isLocationSelected(cityLocation),
                relays: city.relays
                  .map((relay) => {
                    const relayLocation: RelayLocation = {
                      country: country.code,
                      city: city.code,
                      hostname: relay.hostname,
                    };
                    const relayDisabledReason =
                      countryDisabledReason ??
                      cityDisabledReason ??
                      isRelayDisabled(relay, relayLocation, disabledLocation);

                    return {
                      ...relay,
                      label: formatRowName(relay.hostname, relayLocation, relayDisabledReason),
                      location: relayLocation,
                      disabled: relayDisabledReason !== undefined,
                      disabledReason: relayDisabledReason,
                      selected: isLocationSelected(relayLocation),
                    };
                  })
                  .sort((a, b) => a.hostname.localeCompare(b.hostname, locale, { numeric: true })),
              };
            })
            .sort((a, b) => a.label.localeCompare(b.label, locale)),
        };
      })
      .sort((a, b) => a.label.localeCompare(b.label, locale));
  }, [locale, expandedLocations, relayList, disabledLocation, isLocationSelected]);
}

export function usePreventDueToCustomBridgeSelected(): boolean {
  const relaySettings = useNormalRelaySettings();
  const { locationType } = useSelectLocationContext();
  const bridgeSettings = useSelector((state) => state.settings.bridgeSettings);
  const isBridgeSelection =
    relaySettings?.tunnelProtocol === 'openvpn' && locationType === LocationType.entry;

  return isBridgeSelection && bridgeSettings.type === 'custom';
}

// Return all RelayLocations that should be expanded
function useExpandedLocations(filteredLocations: Array<IRelayLocationCountryRedux>) {
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
    [scrollIntoView, spacePreAllocationViewRef],
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

  const expandLocationsForSearch = useEffectEvent(
    (filteredLocations: Array<IRelayLocationCountryRedux>) => {
      if (searchTerm !== '') {
        setExpandedLocations((expandedLocations) => ({
          ...expandedLocations,
          [locationType]: getLocationsExpandedBySearch(filteredLocations, searchTerm),
        }));
      }
    },
  );

  // Expand locations when filters are changed
  useEffect(() => expandLocationsForSearch(filteredLocations), [filteredLocations]);

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
export function useDisabledLocation() {
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
export function useSelectedLocation(): RelayLocation | undefined {
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
      return bridgeSettings?.location === 'any' ? undefined : bridgeSettings?.location;
    }
  }, [
    locationType,
    relaySettings?.location,
    relaySettings?.tunnelProtocol,
    relaySettings?.wireguard.entryLocation,
    bridgeSettings?.location,
  ]);
}
