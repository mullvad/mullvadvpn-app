import type { LocationType } from '../../../../../features/locations/types';
import { Accordion } from '../../../../../lib/components/accordion';
import { FlexColumn } from '../../../../../lib/components/flex-column';
import { useHasCustomLists } from '../../hooks';
import { CountryLocations } from '../country-locations';
import { CustomListLocations } from '../custom-list-locations';
import { NoSearchResult } from '../no-search-result';
import { RecentLocations } from '../recent-locations';
import { useHasSearched, useHasSearchedLocations, useHasVisibleRecentLocations } from './hooks';
import { LocationListsProvider } from './LocationListsContext';

export type LocationsListsProps = React.PropsWithChildren & {
  type: LocationType;
};

export function LocationLists(props: LocationsListsProps) {
  const hasSearched = useHasSearched();
  const hasVisibleRecentLocations = useHasVisibleRecentLocations();
  const hasVisibleCustomLists = useHasCustomLists();
  const hasSearchedLocations = useHasSearchedLocations();

  const showRecentLocations = hasVisibleRecentLocations;
  const showCustomListLocationLists = !hasSearched || hasVisibleCustomLists;
  const showCountryLocations = !hasSearched || hasSearchedLocations;
  const showNoSearchResult =
    hasSearched && !showCustomListLocationLists && !showCountryLocations && !showRecentLocations;

  return (
    <LocationListsProvider {...props}>
      <Accordion expanded={showRecentLocations}>
        <Accordion.Content>
          <RecentLocations />
        </Accordion.Content>
      </Accordion>
      <FlexColumn gap="large">
        {showCustomListLocationLists && <CustomListLocations />}
        {showCountryLocations && <CountryLocations />}
        {showNoSearchResult && <NoSearchResult />}
      </FlexColumn>
    </LocationListsProvider>
  );
}
