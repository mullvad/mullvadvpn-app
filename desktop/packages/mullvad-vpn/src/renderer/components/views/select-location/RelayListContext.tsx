import React, { useCallback, useContext, useEffect, useMemo, useState } from 'react';

import {
  compareRelayLocation,
  ObfuscationType,
  RelayLocation as DaemonRelayLocation,
} from '../../../../shared/daemon-rpc-types';
import {
  filterLocations,
  filterLocationsByDaita,
  filterLocationsByLwo,
  filterLocationsByQuic,
  getLocationsExpandedBySearch,
  searchForLocations,
} from '../../../lib/filter-locations';
import { useNormalRelaySettings } from '../../../lib/relay-settings-hooks';
import { useEffectEvent } from '../../../lib/utility-hooks';
import { IRelayLocationCountryRedux } from '../../../redux/settings/reducers';
import { useSelector } from '../../../redux/store';
import { useDisabledLocation, useSelectedLocation } from './hooks';
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
  type AnyLocation,
  type CityLocation,
  type CountryLocation,
  DisabledReason,
  LocationType,
  type RelayLocation,
  RelayLocationCountryWithVisibility,
} from './select-location-types';
import { useSelectLocationViewContext } from './SelectLocationViewContext';

// Context containing the relay list and related data and callbacks
interface RelayListContext {
  relayList: CountryLocation[];
  expandedLocations?: Array<DaemonRelayLocation>;
  expandLocation: (location: AnyLocation) => void;
  collapseLocation: (location: AnyLocation) => void;
  onBeforeExpand: (
    locationRect: DOMRect,
    expandedContentHeight: number,
    invokedByUser: boolean,
  ) => void;
  expandSearchResults: (searchTerm: string) => void;
}

type ExpandedLocations = Partial<Record<LocationType, Array<DaemonRelayLocation>>>;

export const relayListContext = React.createContext<RelayListContext | undefined>(undefined);

export function useRelayListContext() {
  return useContext(relayListContext)!;
}

interface RelayListContextProviderProps {
  children: React.ReactNode;
}

export function RelayListContextProvider(props: RelayListContextProviderProps) {
  const { locationType, searchTerm } = useSelectLocationViewContext();
  const daita = useSelector((state) => state.settings.wireguard.daita?.enabled ?? false);
  const directOnly = useSelector((state) => state.settings.wireguard.daita?.directOnly ?? false);
  const quic = useSelector(
    (state) => state.settings.obfuscationSettings.selectedObfuscation === ObfuscationType.quic,
  );
  const lwo = useSelector(
    (state) => state.settings.obfuscationSettings.selectedObfuscation === ObfuscationType.lwo,
  );

  const fullRelayList = useSelector((state) => state.settings.relayLocations);
  const relaySettings = useNormalRelaySettings();
  const multihop = relaySettings?.wireguard.useMultihop ?? false;
  const ipVersion = relaySettings?.wireguard.ipVersion ?? 'any';

  const relayListForDaita = useMemo(() => {
    return filterLocationsByDaita(fullRelayList, daita, directOnly, locationType, multihop);
  }, [fullRelayList, daita, directOnly, locationType, multihop]);

  // Only show relays that have QUIC endpoints when QUIC obfuscation is enabled.
  const relayListForQuic = useMemo(() => {
    return filterLocationsByQuic(relayListForDaita, quic, locationType, multihop, ipVersion);
  }, [quic, relayListForDaita, locationType, multihop, ipVersion]);
  // Only show relays that have LWO endpoints when LWO is enabled.
  const relayListForLwo = useMemo(() => {
    return filterLocationsByLwo(relayListForQuic, lwo, locationType, multihop);
  }, [lwo, relayListForQuic, locationType, multihop]);

  // Filters the relays to only keep the relays matching the currently selected filters, e.g.
  // ownership and providers
  const relayListForFilters = useMemo(() => {
    return filterLocations(relayListForLwo, relaySettings?.ownership, relaySettings?.providers);
  }, [relaySettings?.ownership, relaySettings?.providers, relayListForLwo]);

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
      expandedLocations,
      expandLocation,
      collapseLocation,
      onBeforeExpand,
      expandSearchResults,
    }),
    [
      relayList,
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
  expandedLocations?: Array<DaemonRelayLocation>,
): CountryLocation[] {
  const locale = useSelector((state) => state.userInterface.locale);
  const selectedLocation = useSelectedLocation();
  const disabledLocation = useDisabledLocation();

  const isLocationSelected = useCallback(
    (location: DaemonRelayLocation) => {
      return isSelected(location, selectedLocation);
    },
    [selectedLocation],
  );

  return useMemo(() => {
    return relayList
      .map((country) => {
        const countryLocation = { country: country.code };
        const countryDisabledReason = isCountryDisabled(country, countryLocation, disabledLocation);

        const cities = country.cities
          .map((city) => {
            const relays = city.relays
              .map((relay) => {
                const relayLocation: DaemonRelayLocation = {
                  country: country.code,
                  city: city.code,
                  hostname: relay.hostname,
                };
                const relayDisabledReason =
                  countryDisabledReason ??
                  isCityDisabled(
                    city,
                    { country: country.code, city: city.code },
                    disabledLocation,
                  ) ??
                  isRelayDisabled(relay, relayLocation, disabledLocation);

                const mappedRelay: RelayLocation = {
                  type: 'relay',
                  label: formatRowName(relay.hostname, relayLocation, relayDisabledReason),
                  details: {
                    country: country.code,
                    city: city.code,
                    hostname: relay.hostname,
                  },
                  active: relayDisabledReason !== DisabledReason.inactive,
                  disabled: relayDisabledReason !== undefined,
                  disabledReason: relayDisabledReason,
                  expanded: isExpanded(relayLocation, expandedLocations),
                  selected: isLocationSelected(relayLocation),
                  visible: relay.visible,
                };
                return mappedRelay;
              })
              .sort((a, b) => a.label.localeCompare(b.label, locale, { numeric: true }));

            const cityLocation: DaemonRelayLocation = { country: country.code, city: city.code };
            const cityDisabledReason =
              countryDisabledReason ?? isCityDisabled(city, cityLocation, disabledLocation);

            const mappedCity: CityLocation = {
              type: 'city',
              label: formatRowName(city.name, cityLocation, cityDisabledReason),
              details: {
                city: city.code,
                country: country.code,
              },
              active: cityDisabledReason !== DisabledReason.inactive,
              disabled: cityDisabledReason !== undefined,
              disabledReason: cityDisabledReason,
              expanded: isExpanded(cityLocation, expandedLocations),
              selected: isLocationSelected(cityLocation),
              visible: city.visible,
              relays,
            };
            return mappedCity;
          })
          .sort((a, b) => a.label.localeCompare(b.label, locale));

        const mappedCountry: CountryLocation = {
          type: 'country',
          label: formatRowName(country.name, countryLocation, countryDisabledReason),
          details: {
            country: country.code,
          },
          active: countryDisabledReason !== DisabledReason.inactive,
          disabled: countryDisabledReason !== undefined,
          disabledReason: countryDisabledReason,
          expanded: isExpanded(countryLocation, expandedLocations),
          selected: isLocationSelected(countryLocation),
          visible: country.visible,
          cities,
        };

        return mappedCountry;
      })
      .sort((a, b) => a.label.localeCompare(b.label, locale));
  }, [locale, expandedLocations, relayList, disabledLocation, isLocationSelected]);
}

// Return all RelayLocations that should be expanded
function useExpandedLocations(filteredLocations: Array<IRelayLocationCountryRedux>) {
  const { locationType, searchTerm } = useSelectLocationViewContext();
  const { spacePreAllocationViewRef, scrollIntoView } = useScrollPositionContext();
  const relaySettings = useNormalRelaySettings();

  // Keeps the state of which locations are expanded for which locationType. This is used to restore
  // the state when switching back and forth between entry and exit.
  const [expandedLocationsMap, setExpandedLocations] = useState<ExpandedLocations>(() =>
    defaultExpandedLocations(relaySettings),
  );

  const expandLocation = useCallback(
    (location: AnyLocation) => {
      setExpandedLocations((expandedLocations) => ({
        ...expandedLocations,
        [locationType]: [...(expandedLocations[locationType] ?? []), location],
      }));
    },
    [locationType],
  );

  const collapseLocation = useCallback(
    (location: AnyLocation) => {
      setExpandedLocations((expandedLocations) => ({
        ...expandedLocations,
        [locationType]: expandedLocations[locationType]!.filter(
          (item) => !compareRelayLocation(location.details, item),
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
        setExpandedLocations(defaultExpandedLocations(relaySettings));
      } else {
        setExpandedLocations((expandedLocations) => ({
          ...expandedLocations,
          [locationType]: getLocationsExpandedBySearch(filteredLocations, searchTerm),
        }));
      }
    },
    [relaySettings, locationType, filteredLocations],
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
  // These lint rules are disabled for now because the react plugin for eslint does
  // not understand that useEffectEvent should not be added to the dependency array.
  // Enable these rules again when eslint can lint useEffectEvent properly.
  // eslint-disable-next-line react-compiler/react-compiler
  // eslint-disable-next-line react-hooks/exhaustive-deps
  useEffect(() => expandLocationsForSearch(filteredLocations), [filteredLocations]);

  return {
    expandedLocations: expandedLocationsMap[locationType],
    expandLocation,
    collapseLocation,
    onBeforeExpand,
    expandSearchResults,
  };
}
