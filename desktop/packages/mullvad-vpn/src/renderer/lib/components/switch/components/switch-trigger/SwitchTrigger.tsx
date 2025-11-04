import React from 'react';
import styled, { css } from 'styled-components';

import { colors } from '../../../../foundations';
import { useSwitchContext } from '../../';
import { StyledSwitchThumbIndicator } from '../switch-thumb';

export type SwitchTriggerProps = React.ComponentPropsWithRef<'button'>;

export const StyledSwitchTrigger = styled.button<{ $checked?: boolean }>`
  ${({ $checked }) => {
    return css`
      --transition-duration: 0.15s;
      --scale: 1;

      background-color: transparent;
      width: fit-content;

      ${StyledSwitchThumbIndicator} {
        transform-origin: center;
        transition:
          transform 150ms ease,
          background-color 150ms linear;
        transform: translateX(${$checked ? '12px' : '0px'}) scale(var(--scale));
      }

      &&:not(:disabled):hover {
        --scale: calc(14 / 12);
      }

      &&:focus-visible {
        outline: 2px solid ${colors.white};
        outline-offset: -1px;
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
