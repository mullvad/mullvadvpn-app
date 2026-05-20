import React from 'react';
import styled, { css } from 'styled-components';

import { colors } from '../../../../../../../../../../foundations';
import { TextField } from '../../../../../../../../../text-field';
import type { TextFieldInputProps } from '../../../../../../../../../text-field/components';
import { useIsLocationSelected } from '../../../../hooks';
import { useLocationSelectorItemContext } from '../../../../LocationSelectorItemContext';
import { LocationSelectorInputIcon } from './components';

export type LocationSelectorTextFieldInputProps = TextFieldInputProps;

export const StyledInputContainer = styled.div`
  position: relative;
`;

export const StyledTextFieldInput = styled(TextField.Input)<{ $selected: boolean }>`
  ${({ $selected }) => {
    return css`
      background-color: ${colors.darkerBlue10};
      ${() => {
        if ($selected) {
          return css`
            background-color: ${colors.blue40};
          `;
        }
        return null;
      }}
    `;
  }}
`;

export function LocationSelectorTextFieldInput(props: LocationSelectorTextFieldInputProps) {
  const { inputRef, setSelected, id, onSelectedItemChange, onItemInputChange, setInputFocused } =
    useLocationSelectorItemContext();

  const handleFocus = React.useCallback(() => {
    onSelectedItemChange?.(id);
    setInputFocused(true);
  }, [id, onSelectedItemChange, setInputFocused]);

  const selected = useIsLocationSelected(id);

  const handleBlur = React.useCallback(() => {
    setSelected(false);
    setInputFocused(false);
  }, [setSelected, setInputFocused]);

  const handleChange = React.useCallback(
    (event: React.ChangeEvent<HTMLInputElement>) => {
      onItemInputChange?.(id, event.target.value);
    },
    [id, onItemInputChange],
  );

  return (
    <StyledInputContainer>
      <LocationSelectorInputIcon />
      <StyledTextFieldInput
        ref={inputRef}
        onFocus={handleFocus}
        onBlur={handleBlur}
        tabIndex={-1}
        onChange={handleChange}
        $selected={selected}
        {...props}
      />
    </StyledInputContainer>
  );
}
