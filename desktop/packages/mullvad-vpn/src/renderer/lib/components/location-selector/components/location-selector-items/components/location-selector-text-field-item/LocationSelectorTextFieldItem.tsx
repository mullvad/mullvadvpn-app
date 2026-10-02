import type { HTMLMotionProps } from 'motion/react';
import React from 'react';
import styled from 'styled-components';

import { spacings } from '../../../../../../foundations';
import { type LocationSelectorSelectedItem } from '../../../../LocationSelector';
import type { LocationSelectorItemType } from '../../types';
import { LocationSelectorItem } from '../location-selector-item';
import {
  LocationSelectorTextField,
  LocationSelectorTrailingButton,
  LocationSelectorTrigger,
  StyledLocationSelectorTextField,
  StyledLocationSelectorTrailingButton,
} from './components';
import { LocationSelectorTextFieldItemProvider } from './LocationSelectorTextFieldItemContext';

export const StyledLocationSelectorTrigger = styled(LocationSelectorTrigger)`
  width: 100%;
  // Add space between text field and trailing button
  ${StyledLocationSelectorTextField} + ${StyledLocationSelectorTrailingButton} {
    margin-left: ${spacings.tiny};
  }
`;

export type LocationSelectorTextFieldItemProps = Omit<HTMLMotionProps<'div'>, 'children'> & {
  id: LocationSelectorSelectedItem;
  type: LocationSelectorItemType;
  inputRef?: React.RefObject<HTMLInputElement | null>;
  triggerRef?: React.RefObject<HTMLDivElement | null>;
} & React.PropsWithChildren;

function LocationSelectorTextFieldItemImpl({
  children,
  ...props
}: Omit<LocationSelectorTextFieldItemProps, 'id' | 'type'>) {
  return (
    <LocationSelectorItem {...props}>
      <StyledLocationSelectorTrigger>{children}</StyledLocationSelectorTrigger>
    </LocationSelectorItem>
  );
}

function LocationSelectorTextFieldItem({
  id,
  type,
  inputRef,
  triggerRef,
  ...props
}: LocationSelectorTextFieldItemProps) {
  return (
    <LocationSelectorTextFieldItemProvider
      id={id}
      type={type}
      inputRef={inputRef}
      triggerRef={triggerRef}>
      <LocationSelectorTextFieldItemImpl {...props} />
    </LocationSelectorTextFieldItemProvider>
  );
}

const LocationSelectorTextFieldItemNamespace = Object.assign(LocationSelectorTextFieldItem, {
  TextField: LocationSelectorTextField,
  TrailingButton: LocationSelectorTrailingButton,
});

export { LocationSelectorTextFieldItemNamespace as LocationSelectorTextFieldItem };
