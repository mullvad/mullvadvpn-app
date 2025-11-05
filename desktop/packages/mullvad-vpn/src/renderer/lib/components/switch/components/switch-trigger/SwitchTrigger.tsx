import React from 'react';
import styled, { css } from 'styled-components';

import { colors } from '../../../../foundations';
import { useSwitchContext } from '../../';
import { StyledSwitchThumbIndicator } from '../switch-thumb';

export type SwitchTriggerProps = React.ComponentPropsWithRef<'button'>;

export const StyledSwitchTrigger = styled.button<{ $checked?: boolean }>`
  ${({ $checked }) => {
    return css`
      --transition-duration: 0.1s;
      --scale: 1;

      background-color: transparent;
      width: fit-content;
      border-radius: 100px;

      ${StyledSwitchThumbIndicator} {
        transform-origin: center;
        transition:
          transform var(--transition-duration) ease-out,
          background-color var(--transition-duration) linear;
        transform: translateX(${$checked ? '12px' : '0px'}) scale(var(--scale));
      }

      &&:not(:disabled):hover {
        // Scale takes a unitless ratio, so we derive it from the 12px base: 14 / 12 = ~1.1667
        --scale: calc(14 / 12);
      }

      &&:focus-visible {
        outline: 2px solid ${colors.white};
        outline-offset: 2px;
      }
    `;
  }}
`;

export function SwitchTrigger(props: SwitchTriggerProps) {
  const { labelId, checked, disabled, onCheckedChange } = useSwitchContext();
  const handleClick = React.useCallback(() => {
    if (onCheckedChange) {
      onCheckedChange(!checked);
    }
  }, [checked, onCheckedChange]);

  return (
    <StyledSwitchTrigger
      onClick={handleClick}
      disabled={disabled}
      role="switch"
      $checked={checked}
      aria-checked={checked ? 'true' : 'false'}
      aria-labelledby={labelId}
      {...props}
    />
  );
}
