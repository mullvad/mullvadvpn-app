import React from 'react';

import { useIsLocationSelectorExpanded } from './use-is-location-selector-expanded';
import { useIsLocationSelectorIsolated } from './use-is-location-selector-isolated';

export function useHandleMeasure() {
  const [maxHeight, setMaxHeight] = React.useState(0);
  const isLocationSelectorExpanded = useIsLocationSelectorExpanded();
  const isLocationSelectorIsolated = useIsLocationSelectorIsolated();

  const handleMeasure = React.useCallback(
    (height: number) => {
      if (isLocationSelectorExpanded) {
        if (height > maxHeight) {
          setMaxHeight(height);
        }
      }

      if (isLocationSelectorIsolated) {
        return height;
      }

      return maxHeight;
    },
    [isLocationSelectorExpanded, isLocationSelectorIsolated, maxHeight],
  );

  return handleMeasure;
}
