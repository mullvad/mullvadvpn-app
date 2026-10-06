import { LayoutGroup } from 'motion/react';
import React from 'react';
import styled from 'styled-components';

import { FlexColumn } from '../flex-column';
import { LocationSelectorItems, LocationSelectorRow } from './components';
import { LocationSelectorProvider } from './LocationSelectorContext';

export type LocationSelectorPositions = 'top' | 'middle' | 'bottom';
export type LocationSelectorVariant = 'primary' | 'secondary';

export type LocationSelectorSelectedItem = 'entry' | 'exit' | 'automaticEntry';

export type LocationSelectorProps = React.PropsWithChildren<{
  expanded?: boolean;
  selectedItem?: LocationSelectorSelectedItem;
  onSelectedItemChange?: (itemId: LocationSelectorSelectedItem) => void;
  variant: LocationSelectorVariant;
}>;

export const StyledLocationSelector = styled(FlexColumn)`
  --location-selector-z-index: 10;
  --location-selector-line-z-index: var(--location-selector-z-index);
  --location-selector-above-line-z-index: 11;

  position: relative;
  z-index: var(--location-selector-z-index);
`;

function LocationSelector({
  children,
  selectedItem,
  expanded,
  onSelectedItemChange,
  variant,
}: LocationSelectorProps) {
  return (
    <LocationSelectorProvider
      selectedItem={selectedItem}
      onSelectedItemChange={onSelectedItemChange}
      expanded={expanded}
      variant={variant}>
      <LayoutGroup>
        <StyledLocationSelector>{children}</StyledLocationSelector>
      </LayoutGroup>
    </LocationSelectorProvider>
  );
}

const LocationSelectorNamespace = Object.assign(LocationSelector, {
  Items: LocationSelectorItems,
  Row: LocationSelectorRow,
});

export { LocationSelectorNamespace as LocationSelector };
