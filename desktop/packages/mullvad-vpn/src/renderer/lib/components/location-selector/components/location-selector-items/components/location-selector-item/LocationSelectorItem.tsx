import React from 'react';
import styled from 'styled-components';

import { spacings } from '../../../../../../foundations';
import { AnimatedList } from '../../../../../animated-list';
import type { AnimatedListItemProps } from '../../../../../animated-list/components';
import {
  LocationSelectorTextField,
  LocationSelectorTrailingButton,
  LocationSelectorTrigger,
  StyledLocationSelectorTextField,
  StyledLocationSelectorTrailingButton,
} from './components';
import { LocationSelectorItemProvider } from './LocationSelectorItemContext';

export const StyledLocationSelectorItem = styled(AnimatedList.Item)`
  z-index: var(--line-z-index);
`;

export const StyledLocationSelectorTrigger = styled(LocationSelectorTrigger)`
  margin-bottom: ${spacings.tiny};

  // Add space between text field and trailing button
  ${StyledLocationSelectorTextField} + ${StyledLocationSelectorTrailingButton} {
    margin-left: ${spacings.tiny};
  }
`;

export type LocationSelectorItemType = 'entry' | 'entryAutomatic' | 'exit';

export type LocationSelectorItemProps = Omit<AnimatedListItemProps, 'children'> &
  React.PropsWithChildren<{
    id: string;
    type: LocationSelectorItemType;
  }>;

function LocationSelectorItemImpl({
  children,
  ...props
}: Omit<LocationSelectorItemProps, 'id' | 'type'>) {
  return (
    <StyledLocationSelectorItem {...props}>
      <StyledLocationSelectorTrigger>{children}</StyledLocationSelectorTrigger>
    </StyledLocationSelectorItem>
  );
}

function LocationSelectorItem({ id, type, ...props }: LocationSelectorItemProps) {
  return (
    <LocationSelectorItemProvider id={id} type={type}>
      <LocationSelectorItemImpl {...props} />
    </LocationSelectorItemProvider>
  );
}

const LocationSelectorItemNamespace = Object.assign(LocationSelectorItem, {
  TextField: LocationSelectorTextField,
  TrailingButton: LocationSelectorTrailingButton,
});

export { LocationSelectorItemNamespace as LocationSelectorItem };
