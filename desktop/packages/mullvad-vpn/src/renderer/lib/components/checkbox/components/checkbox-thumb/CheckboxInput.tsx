import React from 'react';
import styled from 'styled-components';

import { colors } from '../../../../foundations';
import { Icon } from '../../../icon';
import { useCheckboxContext } from '../../CheckboxContext';

export type CheckboxInputProps = React.ComponentPropsWithRef<'input'>;

const StyledDiv = styled.div`
  position: relative;
  width: fit-content;
`;

export const StyledCheckmarkIcon = styled(Icon)`
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  color: ${colors.green};
  pointer-events: none;
  width: 12px;
`;

export const StyledCheckboxInput = styled.input`
  appearance: none;

  margin: 0;
  padding: 0;
  border: 0;
  background: none;

  box-sizing: border-box;
  display: inline-block;
  vertical-align: middle;
  padding: 3px;

  width: 18px;
  height: 18px;

  border: 2px solid ${colors.white};
  border-radius: 2px;
  background-color: transparent;

  &:checked {
    background-color: ${colors.white};
  }
`;

export function CheckboxInput(props: CheckboxInputProps) {
  const { inputId, checked, onCheckedChange, descriptionId, disabled } = useCheckboxContext();

  const handleChange = React.useCallback(
    (e: React.ChangeEvent<HTMLInputElement>) => {
      onCheckedChange?.(e.target.checked);
    },
    [onCheckedChange],
  );

  return (
    <StyledDiv>
      <StyledCheckboxInput
        id={inputId}
        type="checkbox"
        checked={checked}
        disabled={disabled}
        onChange={handleChange}
        aria-describedby={descriptionId}
        {...props}
      />
      {checked && <StyledCheckmarkIcon icon="checkbox-checkmark" color="green" aria-hidden />}
    </StyledDiv>
  );
}
