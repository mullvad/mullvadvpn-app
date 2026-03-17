import type { LocationType } from '../../../../../features/locations/types';
import { useHasCustomLists } from '../../hooks';
import { CountryLocationList } from '../country-location-list';
import { CustomListLocationLists } from '../custom-list-location-lists';
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
  const showCountryLocationList = !hasSearched || hasSearchedLocations;
  const showNoSearchResult =
    hasSearched && !showCustomListLocationLists && !showCountryLocationList;

  return (
    <LocationListsProvider {...props}>
      {showCustomListLocationLists && <CustomListLocationLists />}
      {showCountryLocationList && <CountryLocationList />}
      {showNoSearchResult && <NoSearchResult />}
    </LocationListsProvider>
  );
}
