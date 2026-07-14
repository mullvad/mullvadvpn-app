import { useRecents } from '../../../../../features/locations/hooks';
import type { LocationType } from '../../../../../features/locations/types';
import { Expandable } from '../../../../../lib/components/expandable';
import { FlexColumn } from '../../../../../lib/components/flex-column';
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

  return (
    <>
      <Expandable expanded={showRecentLocations}>
        <Expandable.Content>
          <RecentLocations />
        </Expandable.Content>
      </Expandable>
      <FlexColumn flexGrow={1} gap="large">
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
