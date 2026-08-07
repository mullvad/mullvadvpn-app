import React from 'react';
import styled, { css } from 'styled-components';

import { colors, Radius, spacings } from '../../foundations';
import { TransientProps } from '../../types';
import { ButtonProvider } from './ButtonContext';
import { ButtonIcon, ButtonText, StyledButtonIcon, StyledButtonText } from './components';

export type ButtonColors = 'neutral' | 'success' | 'destructive';
export type ButtonVariants = 'primary' | 'secondary';

export type ButtonProps = React.ComponentPropsWithRef<'button'> & {
  color?: ButtonColors;
  variant?: ButtonVariants;
  width?: 'fill' | 'fit';
};

const styles = {
  radius: Radius.radius4,
  variants: {
    neutral: {
      color: colors.blue,
      hover: colors.blue60,
      pressed: colors.blue40,
      disabled: colors.blue40,
    },
    success: {
      color: colors.green,
      hover: colors.green80,
      pressed: colors.green40,
      disabled: colors.green40,
    },
    destructive: {
      color: colors.red,
      hover: colors.red80,
      pressed: colors.red40,
      disabled: colors.red40,
    },
  },
};

type StyledButtonProps = TransientProps<Required<Pick<ButtonProps, 'variant' | 'color' | 'width'>>>;

export const StyledButton = styled.button<StyledButtonProps>`
  ${({ $width, $color, $variant }) => {
    const variant = styles.variants[$color];
    const backgroundOrBorderColor = $variant === 'primary' ? 'background' : 'border-color';

    return css`
      --color: ${variant.color};
      --hover: ${variant.hover};
      --pressed: ${variant.pressed};
      --disabled: ${variant.disabled};
      --radius: ${styles.radius};
      --transition-duration: 0.15s;

      display: flex;
      align-items: center;
      padding: ${spacings.tiny} ${spacings.small};
      gap: ${spacings.small};
      overflow-wrap: anywhere;

      min-height: 32px;
      min-width: 60px;
      border-radius: var(--radius);

      ${backgroundOrBorderColor}: var(--color);

      // Add border if secondary appearance
      ${() => {
        if ($variant === 'secondary') {
          return css`
            border: 1px solid var(--color);
            background: transparent;
          `;
        }
        return null;
      }}

      ${() => {
        if ($width === 'fill') {
          return css`
            width: 100%;
          `;
        } else if ($width === 'fit') {
          return css`
            width: fit-content;
            max-width: 100%;
          `;
        }
        return null;
      }}

      @media (prefers-reduced-motion: no-preference) {
        transition: ${backgroundOrBorderColor} var(--transition-duration) ease;
      }

      &&:not(:disabled):hover {
        --transition-duration: 0s;
        ${backgroundOrBorderColor}: var(--hover);
      }

      &&:not(:disabled):active {
        --transition-duration: 0s;
        ${backgroundOrBorderColor}: var(--pressed);
      }

      &:disabled {
        ${backgroundOrBorderColor}: var(--disabled);
      }

      &:focus-visible {
        outline: 2px solid ${colors.white};
        outline-offset: 2px;
      }

      justify-content: space-between;
      &&:has(${StyledButtonText}:only-child) {
        justify-content: center;
      }
      &&:has(${StyledButtonText} + ${StyledButtonIcon}) {
        &::before {
          content: ' ';
          display: inline-block;
          width: 24px;
        }
      }
      &&:has(${StyledButtonIcon} + ${StyledButtonText}) {
        &::after {
          content: ' ';
          display: inline-block;
          width: 24px;
        }
      }
      &&:has(${StyledButtonIcon} + ${StyledButtonText} + ${StyledButtonIcon}) {
        &::before {
          display: none;
        }
        &::after {
          display: none;
        }
      }
    `;
  }}
`;

function Button({
  children,
  disabled = false,
  variant = 'primary',
  color = 'neutral',
  width = 'fill',
  ...props
}: ButtonProps) {
  return (
    <ButtonProvider disabled={disabled}>
      <StyledButton disabled={disabled} $variant={variant} $color={color} $width={width} {...props}>
        {children}
      </StyledButton>
    </ButtonProvider>
  );
}

const ButtonNamespace = Object.assign(Button, {
  Text: ButtonText,
  Icon: ButtonIcon,
});

export { ButtonNamespace as Button };
