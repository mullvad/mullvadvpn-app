import React from 'react';

import { useRecents } from '../../../../../features/locations/hooks';
import type { LocationType } from '../../../../../features/locations/types';
import { FlexColumn } from '../../../../../lib/components/flex-column';
import { useMounted } from '../../../../../lib/utility-hooks';
import { useScrollPositionContext } from '../../ScrollPositionContext';
import { CountryLocations } from '../country-locations';
import { CustomListLocations } from '../custom-list-locations';
import { NoSearchResult } from '../no-search-result';
import { RecentLocations } from '../recent-locations';
import { useHasCustomLists, useHasSearched, useHasSearchedLocations } from './hooks';
import { LocationListsProvider } from './LocationListsContext';

export type LocationsListsProps = {
  type: LocationType;
};

function LocationsListsImpl() {
  const { hasRecents } = useRecents();
  const hasSearched = useHasSearched();
  const hasVisibleCustomLists = useHasCustomLists();
  const hasSearchedLocations = useHasSearchedLocations();

  const showRecentLocations = !hasSearched && hasRecents;
  const showCustomListLocationLists = !hasSearched || hasVisibleCustomLists;
  const showCountryLocations = !hasSearched || hasSearchedLocations;
  const showNoSearchResult =
    hasSearched && !showCustomListLocationLists && !showCountryLocations && !showRecentLocations;

  const { resetScroll } = useScrollPositionContext();

  const mounted = useMounted();
  const isMounted = mounted();

  React.useEffect(() => {
    if (!isMounted) {
      resetScroll();
    }
  }, [resetScroll, isMounted]);

  return (
    <>
      <FlexColumn gap="large">
        {showRecentLocations && <RecentLocations />}
        {showCustomListLocationLists && <CustomListLocations />}
        {showCountryLocations && <CountryLocations />}
        {showNoSearchResult && <NoSearchResult />}
      </FlexColumn>
    </>
  );
}

export function LocationLists({ type }: LocationsListsProps) {
  return (
    <LocationListsProvider type={type}>
      <LocationsListsImpl />
    </LocationListsProvider>
  );
}
