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
  const { inputRef, id, onSelectedItemChange, focusInsideTextField } =
    useLocationSelectorItemContext();

  const handleFocus = React.useCallback(() => {
    onSelectedItemChange?.(id);
  }, [id, onSelectedItemChange]);

  const selected = useIsLocationSelected(id);

  const tabIndex = focusInsideTextField ? 0 : -1;

  return (
    <StyledInputContainer>
      <LocationSelectorInputIcon />
      <StyledTextFieldInput
        ref={inputRef}
        onFocus={handleFocus}
        tabIndex={tabIndex}
        $selected={selected}
        {...props}
      />
    </StyledInputContainer>
  );
}
