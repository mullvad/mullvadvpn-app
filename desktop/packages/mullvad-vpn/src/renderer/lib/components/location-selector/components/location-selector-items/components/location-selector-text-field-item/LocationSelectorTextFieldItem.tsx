import type { HTMLMotionProps } from 'motion/react';
import React from 'react';
import styled from 'styled-components';

import { type LocationSelectorSelectedItem } from '../../../../LocationSelector';
import type { LocationSelectorItemType } from '../../types';
import { LocationSelectorItem } from '../location-selector-item';
import {
  LocationSelectorTextField,
  LocationSelectorTrailingButton,
  LocationSelectorTrigger,
} from './components';
import { LocationSelectorTextFieldItemProvider } from './LocationSelectorTextFieldItemContext';

export const StyledLocationSelectorTrigger = styled(LocationSelectorTrigger)`
  width: 100%;
`;

export type LocationSelectorTextFieldItemProps = Omit<HTMLMotionProps<'div'>, 'children'> & {
  id: LocationSelectorSelectedItem;
  type: LocationSelectorItemType;
  inputRef?: React.RefObject<HTMLInputElement | null>;
  triggerRef?: React.RefObject<HTMLDivElement | null>;
  validate?: (value: string) => boolean | string;
} & React.PropsWithChildren;

function LocationSelectorTextFieldItemImpl({
  children,
  ...props
}: Omit<LocationSelectorTextFieldItemProps, 'id' | 'type'>) {
  return <LocationSelectorItem {...props}>{children}</LocationSelectorItem>;
}

function LocationSelectorTextFieldItem({
  id,
  type,
  inputRef,
  triggerRef,
  validate,
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
  Trigger: StyledLocationSelectorTrigger,
});

export { LocationSelectorTextFieldItemNamespace as LocationSelectorTextFieldItem };
