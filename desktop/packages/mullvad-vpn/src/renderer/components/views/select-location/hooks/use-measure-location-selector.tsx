import React from 'react';
import styled from 'styled-components';

import { SelectLocationHeader, SelectLocationSelector } from '../components';
import { useIsLocationSelectorIsolated } from './use-is-location-selector-isolated';
import { useLocationSelectorItems } from './use-location-selector-items';
import { useShowFilterChips } from './use-show-filter-chips';

const StyledMeasureElementsContainer = styled.div`
  position: absolute;
  visibility: hidden;
`;

export function useMeasureLocationSelector() {
  const ref = React.useRef<HTMLDivElement>(null);
  const collapsedRef = React.useRef<HTMLDivElement>(null);
  const [height, setHeight] = React.useState(0);
  const [collapsedHeight, setCollapsedHeight] = React.useState(0);
  const items = useLocationSelectorItems('measure');

  const isolated = useIsLocationSelectorIsolated();
  // This condition differs from expanded state in real location selector since
  // it should not take scroll position into account.
  const expanded = !isolated;

  const showFilterChips = useShowFilterChips();

  const measure = React.useCallback(() => {
    if (ref.current) {
      const newHeight = ref.current.getBoundingClientRect().height;
      setHeight(newHeight);
    }
    if (collapsedRef.current) {
      const newCollapsedHeight = collapsedRef.current.getBoundingClientRect().height;
      setCollapsedHeight(newCollapsedHeight);
    }
  }, []);

  React.useLayoutEffect(measure);

  React.useEffect(() => {
    const element = ref.current;
    if (!element) {
      return;
    }
    const observer = new ResizeObserver(measure);
    observer.observe(element);
    return () => {
      observer.disconnect();
    };
  }, [measure]);

  // Change key to force remount and skip animations when content changes.
  const key = `${expanded}-${showFilterChips}-${Object.keys(items).join('-')}`;
  const collapsedKey = `${showFilterChips}-${Object.keys(items).join('-')}`;

  const measureElement = (
    <StyledMeasureElementsContainer inert>
      <div ref={ref}>
        <SelectLocationHeader>
          <SelectLocationSelector key={key} expanded={expanded} showFilterChips={showFilterChips} />
        </SelectLocationHeader>
      </div>
      <div ref={collapsedRef}>
        <SelectLocationHeader>
          <SelectLocationSelector
            key={collapsedKey}
            expanded={false}
            showFilterChips={showFilterChips}
          />
        </SelectLocationHeader>
      </div>
    </StyledMeasureElementsContainer>
  );

  return {
    measureElement,
    height,
    collapsedHeight,
  };
}
