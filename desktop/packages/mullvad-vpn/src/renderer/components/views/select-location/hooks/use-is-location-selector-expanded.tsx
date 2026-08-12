import React from 'react';

import { useSelectLocationViewContext } from '../SelectLocationViewContext';

export function useIsLocationSelectorExpanded(collapsibleContentHeight: number): boolean {
  const { locationSelectorExpanded, setLocationSelectorExpanded, scrollTop } =
    useSelectLocationViewContext();

  // The deadzone is a buffer area that prevents the location selector from rapidly toggling
  // between expanded and collapsed when the user scrolls near the threshold.
  const deadzone = collapsibleContentHeight / 8;
  const collapseTreshold = collapsibleContentHeight + deadzone;
  const expandThreshold = collapseTreshold - collapsibleContentHeight;

  React.useLayoutEffect(() => {
    if (scrollTop > collapseTreshold) {
      setLocationSelectorExpanded(false);
    } else if (scrollTop < expandThreshold) {
      setLocationSelectorExpanded(true);
    }
  }, [scrollTop, collapseTreshold, expandThreshold, setLocationSelectorExpanded]);

  return locationSelectorExpanded;
}
