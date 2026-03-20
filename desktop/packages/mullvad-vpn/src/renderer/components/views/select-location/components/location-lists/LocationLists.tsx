import type { LocationType } from '../../../../../features/locations/types';
import { useHasCustomLists } from '../../hooks';
import { CountryLocations } from '../country-locations';
import { CustomListLocations } from '../custom-list-locations';
import { NoSearchResult } from '../no-search-result';
import { useHasSearched, useHasSearchedLocations } from './hooks';
import { LocationListsProvider } from './LocationListsContext';

export type LocationsListsProps = React.PropsWithChildren & {
  type: LocationType;
};

export function LocationLists(props: LocationsListsProps) {
  const hasSearched = useHasSearched();
  const hasVisibleCustomLists = useHasCustomLists();
  const hasSearchedLocations = useHasSearchedLocations();

  const showCustomListLocationLists = !hasSearched || hasVisibleCustomLists;
  const showCountryLocations = !hasSearched || hasSearchedLocations;
  const showNoSearchResult = hasSearched && !showCustomListLocationLists && !showCountryLocations;

  return (
    <LocationListsProvider {...props}>
      {showCustomListLocationLists && <CustomListLocations />}
      {showCountryLocations && <CountryLocations />}
      {showNoSearchResult && <NoSearchResult />}
    </LocationListsProvider>
  );
}
