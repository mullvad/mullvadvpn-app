import React from 'react';
import styled from 'styled-components';

import { useActiveFilters } from '../../../../features/locations/hooks';
import { LocationSelector } from '../../../../lib/components/location-selector';
import {
  SelectLocationHeader,
  SelectLocationSelectorDeviceRow,
  SelectLocationSelectorInternetRow,
} from '../components';
import { useSelectLocationViewContext } from '../SelectLocationViewContext';
import { useEntryType } from './use-entry-type';

const StyledMeasureElement = styled.div`
  position: absolute;
  visibility: hidden;
`;

export function useMeasureExpandedLocationSelector() {
  const singlehopRef = React.useRef<HTMLDivElement>(null);
  const multihopRef = React.useRef<HTMLDivElement>(null);
  const [height, setHeight] = React.useState(0);
  const { locationType } = useSelectLocationViewContext();
  const entryType = useEntryType();
  const activeFilters = useActiveFilters(locationType);

  const singlehopElement = (
    <StyledMeasureElement ref={singlehopRef} inert>
      <SelectLocationHeader>
        <LocationSelector variant="primary" expanded>
          <SelectLocationSelectorDeviceRow />
          <LocationSelector.Items>
            <LocationSelector.Items.TextFieldItem
              id="exit"
              type="exit"
              key="measure-singlehop-exit">
              <LocationSelector.Items.TextFieldItem.TextField>
                <LocationSelector.Items.TextFieldItem.TextField.Input />
              </LocationSelector.Items.TextFieldItem.TextField>
            </LocationSelector.Items.TextFieldItem>
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
            <LocationSelector.Items.TextFieldItem
              id="entry"
              type="entry"
              key="measure-singlehop-entry">
              <LocationSelector.Items.TextFieldItem.TextField>
                <LocationSelector.Items.TextFieldItem.TextField.Input />
              </LocationSelector.Items.TextFieldItem.TextField>
            </LocationSelector.Items.TextFieldItem>
            <LocationSelector.Items.TextFieldItem
              id="exit"
              type="exit"
              key="measure-singlehop-exit">
              <LocationSelector.Items.TextFieldItem.TextField>
                <LocationSelector.Items.TextFieldItem.TextField.Input />
              </LocationSelector.Items.TextFieldItem.TextField>
            </LocationSelector.Items.TextFieldItem>
          </LocationSelector.Items>
          <SelectLocationSelectorInternetRow />
        </LocationSelector>
      </SelectLocationHeader>
    </StyledMeasureElement>
  );

  // Measure when entryType or locationType changes
  React.useLayoutEffect(() => {
    if (entryType) {
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
  }, [locationType, entryType, activeFilters]);

  return {
    singlehopElement,
    multihopElement,
    height,
  };
}
