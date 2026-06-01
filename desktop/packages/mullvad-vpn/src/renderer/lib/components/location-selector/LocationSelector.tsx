import React from 'react';
import styled from 'styled-components';

import { FlexColumn } from '../flex-column';
import { LocationSelectorItems, LocationSelectorRow } from './components';
import { LocationSelectorProvider } from './LocationSelectorContext';

export type LocationSelectorPositions = 'top' | 'middle' | 'bottom';
export type LocationSelectorVariant = 'primary' | 'secondary';

export type LocationSelectorProps = React.PropsWithChildren<{
  expanded?: boolean;
  selectedItem?: string;
  onSelectedItemChange?: (itemId: string) => void;
  variant: LocationSelectorVariant;
}>;

export const StyledLocationSelector = styled(FlexColumn)`
  --line-z-index: 5;
  --above-line-z-index: 6;
  position: relative;
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
      <StyledLocationSelector>{children}</StyledLocationSelector>
    </LocationSelectorProvider>
  );
}

const LocationSelectorNamespace = Object.assign(LocationSelector, {
  Items: LocationSelectorItems,
  Row: LocationSelectorRow,
});

export { LocationSelectorNamespace as LocationSelector };
