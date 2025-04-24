import React, { forwardRef } from 'react';
import styled, { css } from 'styled-components';

import { Colors, Radius, spacings } from '../../foundations';
import { ButtonBase } from './ButtonBase';
import { ButtonProvider } from './ButtonContext';
import { ButtonIcon, ButtonText, StyledButtonIcon, StyledButtonText } from './components';

export interface ButtonProps extends React.ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: 'primary' | 'success' | 'destructive';
  width?: 'fill' | 'fit';
}

const styles = {
  radius: Radius.radius4,
  variants: {
    primary: {
      background: Colors.blue,
      hover: Colors.blue60,
      disabled: Colors.blue50,
    },
    success: {
      background: Colors.green,
      hover: Colors.green90,
      disabled: Colors.green40,
    },
    destructive: {
      background: Colors.red,
      hover: Colors.red80,
      disabled: Colors.red60,
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

const StyledButton = styled(ButtonBase)<ButtonProps>`
  ${({ width: sizeProp = 'fill', variant: variantProp = 'primary' }) => {
    const variant = styles.variants[variantProp];
    const size = styles.flex[sizeProp];
    const width = styles.widths[sizeProp];

    return css`
      --background: ${variant.background};
      --hover: ${variant.hover};
      --disabled: ${variant.disabled};
      --radius: ${styles.radius};
      --size: ${size};
      --width: ${width};

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

      &:not(:disabled):hover {
        background: var(--hover);
      }

      &:disabled {
        background: var(--disabled);
      }

      &:focus-visible {
        outline: 2px solid ${Colors.white};
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

const ForwardedButton = forwardRef<HTMLButtonElement, ButtonProps>(function Button(
  { children, disabled = false, style, ...props },
  ref,
) {
  return (
    <ButtonProvider disabled={disabled}>
      <StyledButton ref={ref} disabled={disabled} {...props}>
        {children}
      </StyledButton>
    </ButtonProvider>
  );
});

const ButtonNamespace = Object.assign(ForwardedButton, {
  Text: ButtonText,
  Icon: ButtonIcon,
});

export { ButtonNamespace as Button };
