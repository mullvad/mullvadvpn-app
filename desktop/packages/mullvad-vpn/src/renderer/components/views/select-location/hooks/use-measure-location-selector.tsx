import React from 'react';
import styled from 'styled-components';

import { LocationSelector } from '../../../../lib/components/location-selector';
import {
  SelectLocationHeader,
  SelectLocationSelectorDeviceRow,
  SelectLocationSelectorInternetRow,
} from '../components';
import { useSelectLocationViewContext } from '../SelectLocationViewContext';
import { useLocationSelectorItems } from './use-location-selector-items';

const StyledMeasureElement = styled.div`
  position: absolute;
  visibility: hidden;
`;

export function useMeasureLocationSelector() {
  const ref = React.useRef<HTMLDivElement>(null);
  const [height, setHeight] = React.useState(0);
  const { isolatedItem } = useSelectLocationViewContext();
  const items = useLocationSelectorItems('measure');
  const expanded = isolatedItem === undefined;

  const measure = React.useCallback(() => {
    if (ref.current) {
      const newHeight = ref.current.getBoundingClientRect().height;
      setHeight(newHeight);
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
  const key = `${expanded ? 'expanded' : 'isolated'}-${Object.keys(items).join('-')}`;

  const measureElement = (
    <StyledMeasureElement ref={ref} inert>
      <SelectLocationHeader>
        <LocationSelector key={key} variant="primary" expanded={expanded}>
          <SelectLocationSelectorDeviceRow />
          <LocationSelector.Items>{Object.values(items)}</LocationSelector.Items>
          <SelectLocationSelectorInternetRow />
        </LocationSelector>
      </SelectLocationHeader>
    </StyledMeasureElement>
  );

  return {
    measureElement,
    height,
  };
}
