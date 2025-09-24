import React from 'react';
import styled, { css } from 'styled-components';

import { colors, Radius, spacings } from '../../foundations';
import { ButtonProvider } from './ButtonContext';
import { ButtonIcon, ButtonText, StyledButtonIcon, StyledButtonText } from './components';

export type ButtonProps = React.ComponentPropsWithRef<'button'> & {
  variant?: 'primary' | 'success' | 'destructive';
  width?: 'fill' | 'fit';
};

const styles = {
  radius: Radius.radius4,
  variants: {
    primary: {
      background: colors.blue,
      hover: colors.blue60,
      pressed: colors.blue40,
      disabled: colors.blue40,
    },
    success: {
      background: colors.green,
      hover: colors.green80,
      pressed: colors.green40,
      disabled: colors.green40,
    },
    destructive: {
      background: colors.red,
      hover: colors.red80,
      pressed: colors.red40,
      disabled: colors.red40,
    },
  },
  flex: {
    fill: '1 1 0',
    fit: '0 0 auto',
  },
  widths: {
    fill: undefined,
    fit: 'fit-content',
  },
};

export const StyledButton = styled.button<ButtonProps>`
  ${({ width: sizeProp = 'fill', variant: variantProp = 'primary' }) => {
    const variant = styles.variants[variantProp];
    const size = styles.flex[sizeProp];
    const width = styles.widths[sizeProp];

    return css`
      --background: ${variant.background};
      --hover: ${variant.hover};
      --pressed: ${variant.pressed};
      --disabled: ${variant.disabled};
      --radius: ${styles.radius};
      --size: ${size};
      --width: ${width};
      --transition-duration: 0.15s;

      display: flex;
      flex: var(--size);
      align-items: center;
      padding: ${spacings.tiny} ${spacings.small};
      gap: ${spacings.small};

      min-height: 32px;
      min-width: 60px;
      width: var(--width);
      border-radius: var(--radius);
      background: var(--background);

      @media (prefers-reduced-motion: no-preference) {
        transition: background-color var(--transition-duration) ease;
      }

      &&:not(:disabled):hover {
        --transition-duration: 0s;
        background: var(--hover);
      }

      &&:not(:disabled):active {
        --transition-duration: 0s;
        background: var(--pressed);
      }

      &:disabled {
        background: var(--disabled);
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

function Button({ children, disabled = false, ...props }: ButtonProps) {
  return (
    <ButtonProvider disabled={disabled}>
      <StyledButton disabled={disabled} {...props}>
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
