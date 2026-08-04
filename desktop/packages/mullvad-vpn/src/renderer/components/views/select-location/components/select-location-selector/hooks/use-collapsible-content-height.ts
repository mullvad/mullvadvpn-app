import React from 'react';

import { useActiveFilters } from '../../../../../../features/locations/hooks';
import { useSelectLocationViewContext } from '../../../SelectLocationViewContext';

export function useCollapsibleContentHeight(
  topRowRef: React.RefObject<HTMLDivElement | null>,
  bottomRowRef: React.RefObject<HTMLDivElement | null>,
) {
  const [topRowHeight, setTopRowHeight] = React.useState(0);
  const [bottomRowHeight, setBottomRowHeight] = React.useState(0);

  const { locationType, locationSelectorExpanded } = useSelectLocationViewContext();

  const { isAnyFilterActive } = useActiveFilters(locationType);

  React.useLayoutEffect(() => {
    if (topRowRef.current) {
      setTopRowHeight(topRowRef.current.offsetHeight);
    }
    if (bottomRowRef.current) {
      setBottomRowHeight(bottomRowRef.current.offsetHeight);
    }
  }, [bottomRowRef, topRowRef, locationSelectorExpanded, isAnyFilterActive]);

  return topRowHeight + bottomRowHeight;
}
