import React from 'react';
import styled from 'styled-components';

import { spacings } from '../../../../../../../../foundations';
import { TextField, type TextFieldProps } from '../../../../../../../text-field';
import { useLocationSelectorItemContext } from '../../LocationSelectorItemContext';
import {
  LocationSelectorClearButton,
  LocationSelectorTextFieldInput,
  LocationSelectorTextFieldSupportingText,
  StyledLocationSelectorTextFieldInputInput,
} from './components';

export type LocationSelectorTextFieldProps = Omit<TextFieldProps, 'onValueChange'> & {
  onValueChange?: (id: string, value: string) => void;
  onFocusExit?: () => void;
};

export const StyledLocationSelectorTextField = styled(TextField)`
  ${StyledLocationSelectorTextFieldInputInput} {
    padding-left: calc(${spacings.small} + 18px + ${spacings.tiny});
  }
`;

function LocationSelectorTextField({
  value,
  onValueChange,
  onFocusExit,
  ...props
}: LocationSelectorTextFieldProps) {
  const { id, textFieldRef, inputRef, setFocusInsideTextField } = useLocationSelectorItemContext();

  const handleOnValueChange = React.useCallback(
    (value: string) => {
      onValueChange?.(id, value);
    },
    [onValueChange, id],
  );

  const handleOnFocusCapture = React.useCallback(() => {
    setFocusInsideTextField(true);
  }, [setFocusInsideTextField]);

  const handleOnBlurCapture = React.useCallback(
    (e: React.FocusEvent<HTMLDivElement>) => {
      const focusInsideTextField = inputRef.current?.contains(e.relatedTarget) ?? false;
      setFocusInsideTextField(focusInsideTextField);
      if (!focusInsideTextField) {
        onFocusExit?.();
      }
    },
    [inputRef, setFocusInsideTextField, onFocusExit],
  );

  return (
    <StyledLocationSelectorTextField
      ref={textFieldRef}
      variant="secondary"
      value={value}
      onValueChange={handleOnValueChange}
      onFocusCapture={handleOnFocusCapture}
      onBlurCapture={handleOnBlurCapture}
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
