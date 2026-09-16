import React from 'react';
import styled from 'styled-components';

import { LocationSelector } from '../../../../lib/components/location-selector';
import { SelectLocationHeader } from '../components';

const StyledMeasureElement = styled.div`
  position: absolute;
  visibility: hidden;
`;

export function useMeasureIsolatedLocationSelector() {
  const ref = React.useRef<HTMLDivElement>(null);
  const [height, setHeight] = React.useState(0);

  const element = (
    <StyledMeasureElement ref={ref} inert>
      <SelectLocationHeader>
        <LocationSelector variant="primary" expanded={false}>
          <LocationSelector.Items>
            <LocationSelector.Items.Item id="exit" type="exit" key="measure-isolated-item">
              <LocationSelector.Items.Item.TextField>
                <LocationSelector.Items.Item.TextField.Input />
              </LocationSelector.Items.Item.TextField>
            </LocationSelector.Items.Item>
          </LocationSelector.Items>
        </LocationSelector>
      </SelectLocationHeader>
    </StyledMeasureElement>
  );

  // Measure once since height will not change while in view
  React.useLayoutEffect(() => {
    if (ref.current) {
      const newHeight = ref.current.getBoundingClientRect().height;
      setHeight(newHeight);
    }
  }, []);

  return {
    element,
    height,
  };
}
