import React from 'react';
import styled from 'styled-components';

import { useMultihop } from '../../../../features/multihop/hooks';
import { LocationSelector } from '../../../../lib/components/location-selector';
import {
  SelectLocationHeader,
  SelectLocationSelectorDeviceRow,
  SelectLocationSelectorInternetRow,
} from '../components';
import { useSelectLocationViewContext } from '../SelectLocationViewContext';

const StyledMeasureElement = styled.div`
  position: absolute;
  visibility: hidden;
`;

export function useMeasureExpandedLocationSelector() {
  const singlehopRef = React.useRef<HTMLDivElement>(null);
  const multihopRef = React.useRef<HTMLDivElement>(null);
  const [height, setHeight] = React.useState(0);
  const { locationType } = useSelectLocationViewContext();
  const { multihop } = useMultihop();

  const singlehopElement = (
    <StyledMeasureElement ref={singlehopRef} inert>
      <SelectLocationHeader>
        <LocationSelector variant="primary" expanded>
          <SelectLocationSelectorDeviceRow />
          <LocationSelector.Items>
            <LocationSelector.Items.Item id="exit" type="exit" key="measure-singlehop-exit">
              <LocationSelector.Items.Item.TextField>
                <LocationSelector.Items.Item.TextField.Input />
              </LocationSelector.Items.Item.TextField>
            </LocationSelector.Items.Item>
          </LocationSelector.Items>
          <SelectLocationSelectorInternetRow />
        </LocationSelector>
      </SelectLocationHeader>
    </StyledMeasureElement>
  );

  const multihopElement = (
    <StyledMeasureElement ref={multihopRef} inert>
      <SelectLocationHeader>
        <LocationSelector variant="primary" expanded>
          <SelectLocationSelectorDeviceRow />
          <LocationSelector.Items>
            <LocationSelector.Items.Item id="entry" type="entry" key="measure-singlehop-entry">
              <LocationSelector.Items.Item.TextField>
                <LocationSelector.Items.Item.TextField.Input />
              </LocationSelector.Items.Item.TextField>
            </LocationSelector.Items.Item>
            <LocationSelector.Items.Item id="exit" type="exit" key="measure-singlehop-exit">
              <LocationSelector.Items.Item.TextField>
                <LocationSelector.Items.Item.TextField.Input />
              </LocationSelector.Items.Item.TextField>
            </LocationSelector.Items.Item>
          </LocationSelector.Items>
          <SelectLocationSelectorInternetRow />
        </LocationSelector>
      </SelectLocationHeader>
    </StyledMeasureElement>
  );

  // Measure when locationType or multihop changes
  React.useLayoutEffect(() => {
    if (multihop === 'always') {
      if (multihopRef.current) {
        const newHeight = multihopRef.current.offsetHeight;
        setHeight(newHeight);
      }
    } else {
      if (singlehopRef.current) {
        const newHeight = singlehopRef.current.offsetHeight;
        setHeight(newHeight);
      }
    }
  }, [locationType, multihop]);

  return {
    singlehopElement,
    multihopElement,
    height,
  };
}
