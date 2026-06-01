import { motion } from 'motion/react';
import React from 'react';
import styled, { css } from 'styled-components';

import { colors } from '../../../../../../../../../../foundations';
import { TextField } from '../../../../../../../../../text-field';
import type { TextFieldInputProps } from '../../../../../../../../../text-field/components';
import { useIsLocationSelected } from '../../../../hooks';
import { useLocationSelectorItemContext } from '../../../../LocationSelectorItemContext';
import { LocationSelectorInputIcon } from './components';

export type LocationSelectorTextFieldInputProps = TextFieldInputProps;

export const StyledLocationSelectorTextFieldInput = styled.div`
  position: relative;
  display: flex;
  flex: 1;
`;

export const StyledLocationSelectorTextFieldInputInput = styled(TextField.Input)<{
  $selected: boolean;
}>`
  ${({ $selected }) => {
    return css`
      margin-left: 4px;
      margin-right: 4px;
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
export const StyledInputAnimationContainer = styled(motion.div)`
  display: flex;
  flex: 1;
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
    <StyledLocationSelectorTextFieldInput>
      <LocationSelectorInputIcon />
      <StyledInputAnimationContainer layout transition={{ duration: 0.15, ease: 'linear' }}>
        <StyledLocationSelectorTextFieldInputInput
          ref={inputRef}
          onFocus={handleFocus}
          tabIndex={tabIndex}
          $selected={selected}
          {...props}
        />
      </StyledInputAnimationContainer>
    </StyledLocationSelectorTextFieldInput>
  );
}
