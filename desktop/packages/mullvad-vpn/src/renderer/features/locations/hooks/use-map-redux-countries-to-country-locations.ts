import React from 'react';

import type { IRelayLocationCountryRedux } from '../../../redux/settings/reducers';
import { useSelector } from '../../../redux/store';
import { type CountryLocation, LocationType } from '../types';
import { mapReduxCountryToCountryLocation } from '../utils';
import { useDisabledLocation } from './use-disabled-location';
import { useSelectedEntryOrExitLocation } from './use-selected-entry-or-exit-location';

export function useMapReduxCountriesToCountryLocations(
  locationType: LocationType,
  relayList: Array<IRelayLocationCountryRedux>,
): CountryLocation[] {
  const locale = useSelector((state) => state.userInterface.locale);
  const selectedLocation = useSelectedEntryOrExitLocation(locationType);
  const disabledLocation = useDisabledLocation(locationType);
  const matches = useSelector((state) => state.settings.relayLocationsFiltered['exit'].matches);

  return React.useMemo(() => {
    return relayList
      .map((country) =>
        mapReduxCountryToCountryLocation(
          country,
          selectedLocation === 'any' ? undefined : selectedLocation,
          disabledLocation,
          locale,
          matches,
        ),
      )
      .sort((a, b) => a.label.localeCompare(b.label, locale));
  }, [relayList, selectedLocation, disabledLocation, locale, matches]);
}
