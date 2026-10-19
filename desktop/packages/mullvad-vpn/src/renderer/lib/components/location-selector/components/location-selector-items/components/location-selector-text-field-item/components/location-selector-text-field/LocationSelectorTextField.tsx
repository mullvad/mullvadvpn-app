import React from 'react';
import styled from 'styled-components';

import { spacings } from '../../../../../../../../foundations';
import { TextField, type TextFieldProps } from '../../../../../../../text-field';
import { type LocationSelectorSelectedItem } from '../../../../../../LocationSelector';
import { useLocationSelectorTextFieldItemContext } from '../../LocationSelectorTextFieldItemContext';
import {
  LocationSelectorClearButton,
  LocationSelectorTextFieldInput,
  LocationSelectorTextFieldSupportingText,
  StyledLocationSelectorTextFieldInputInput,
} from './components';

export type LocationSelectorTextFieldProps = Omit<TextFieldProps, 'onValueChange'> & {
  onValueChange?: (id: LocationSelectorSelectedItem, value: string) => void;
};

export const StyledLocationSelectorTextField = styled(TextField)`
  ${StyledLocationSelectorTextFieldInputInput} {
    padding-left: calc(${spacings.small} + 18px + ${spacings.tiny});
  }
`;

function LocationSelectorTextField({
  value,
  onValueChange,
  ...props
}: LocationSelectorTextFieldProps) {
  const { id, textFieldRef } = useLocationSelectorTextFieldItemContext();

  const handleOnValueChange = React.useCallback(
    (value: string) => {
      onValueChange?.(id, value);
    },
    [onValueChange, id],
  );

  return (
    <StyledLocationSelectorTextField
      ref={textFieldRef}
      variant="secondary"
      value={value}
      onValueChange={handleOnValueChange}
      {...props}
    />
  );
}

const LocationSelectorTextFieldNamespace = Object.assign(LocationSelectorTextField, {
  Input: LocationSelectorTextFieldInput,
  SupportingText: LocationSelectorTextFieldSupportingText,
  ClearButton: LocationSelectorClearButton,
});

export { LocationSelectorTextFieldNamespace as LocationSelectorTextField };
