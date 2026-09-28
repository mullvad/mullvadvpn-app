import React from 'react';

import { useSelectedEntryOrExitLocation } from '../../../../../../features/locations/hooks';
import { LocationType } from '../../../../../../features/locations/types';
import { useLocationInvalidReason } from './use-location-invalid-reason';
import { useLocationName } from './use-location-name';

export function useGetIsLocationValid(locationType: LocationType) {
  const location = useSelectedEntryOrExitLocation(locationType);
  const locationName = useLocationName(location);
  const invalidReason = useLocationInvalidReason(locationType);

  const getIsLocationValid = React.useCallback(
    (value: string) => {
      if (value === locationName) {
        if (invalidReason) {
          return invalidReason;
        }
      }

      return true;
    },
    [locationName, invalidReason],
  );

  return getIsLocationValid;
}
