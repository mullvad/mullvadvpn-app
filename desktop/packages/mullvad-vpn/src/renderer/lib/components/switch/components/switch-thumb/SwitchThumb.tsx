import React from 'react';
import styled, { css } from 'styled-components';

import { colors } from '../../../../foundations';
import { useSwitchContext } from '../../SwitchContext';
import { useBackgroundColor, useBorderColor } from './hooks';

export type SwitchThumbProps = React.ComponentPropsWithRef<'input'>;

export const StyledSwitchThumb = styled.input<{
  $borderColor: string;
  $indicatorColor?: string;
  $checked?: boolean;
}>`
  ${({ $borderColor, $indicatorColor, $checked }) => {
    return css`
      --transition-duration: 0.1s;
      --scale: 1;

      appearance: none;

      margin: 0;
      padding: 0;
      border: 0;
      background: none;

      box-sizing: border-box;
      display: inline-block;
      vertical-align: middle;

      position: relative;
      display: flex;
      align-items: center;
      width: 32px;
      height: 20px;
      border: 2px solid ${$borderColor};
      border-radius: 100px;
      transition: border-color 200ms ease;

      &&:not(:disabled):hover {
        // Scale takes a unitless ratio, so we derive it from the 12px base: 14 / 12 = ~1.1667
        --scale: calc(14 / 12);
      }

      &&:focus-visible {
        outline: 2px solid ${colors.white};
        outline-offset: 2px;
      }

      &&::before {
        content: '';
        position: absolute;
        left: 2px;
        background-color: ${$indicatorColor};

        min-width: 12px;
        width: 12px;
        aspect-ratio: 1 / 1;
        border-radius: 50%;

        transform-origin: center;
        transition:
          transform var(--transition-duration) ease-out,
          background-color var(--transition-duration) linear;
        transform: translateX(${$checked ? '12px' : '0px'}) scale(var(--scale));
      }
    `;
  }}
`;

export function SwitchThumb(props: SwitchThumbProps) {
  const { inputId, checked, onCheckedChange, descriptionId, disabled } = useSwitchContext();
  const backgroundColor = useBackgroundColor();
  const borderColor = useBorderColor();

  const handleChange = React.useCallback(
    (e: React.ChangeEvent<HTMLInputElement>) => {
      onCheckedChange?.(e.target.checked);
    },
    [onCheckedChange],
  );

  return (
    <StyledSwitchThumb
      id={inputId}
      type="checkbox"
      role="switch"
      checked={checked}
      disabled={disabled}
      onChange={handleChange}
      aria-describedby={descriptionId}
      $checked={checked}
      $borderColor={colors[borderColor]}
      $indicatorColor={colors[backgroundColor]}
      {...props}
    />
  );
}
