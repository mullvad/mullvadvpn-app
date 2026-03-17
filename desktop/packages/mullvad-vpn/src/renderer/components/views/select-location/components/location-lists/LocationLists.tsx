import type { LocationType } from '../../../../../features/locations/types';
import { useHasCustomLists } from '../../hooks';
import { CountryLocationList } from '../country-location-list';
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

  const showCustomListLocations = !hasSearched || hasVisibleCustomLists;
  const showCountryLocationList = !hasSearched || hasSearchedLocations;
  const showNoSearchResult = hasSearched && !showCustomListLocations && !showCountryLocationList;

  return (
    <LocationListsProvider {...props}>
      {showCustomListLocations && <CustomListLocations />}
      {showCountryLocationList && <CountryLocationList />}
      {showNoSearchResult && <NoSearchResult />}
    </LocationListsProvider>
  );
}
