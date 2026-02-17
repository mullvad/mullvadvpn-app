import { useHasCustomLists } from '../../hooks';
import type { LocationType } from '../../select-location-types';
import { CountryLocationList } from '../country-location-list';
import { CustomListLocationList } from '../custom-list-location-list';
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

  const showCustomListLocationList = !hasSearched || hasVisibleCustomLists;
  const showCountryLocationList = !hasSearched || hasSearchedLocations;
  const showNoSearchResult = hasSearched && !showCustomListLocationList && !showCountryLocationList;

  return (
    <LocationListsProvider {...props}>
      {showCustomListLocationList && <CustomListLocationList />}
      {showCountryLocationList && <CountryLocationList />}
      {showNoSearchResult && <NoSearchResult />}
    </LocationListsProvider>
  );
}
