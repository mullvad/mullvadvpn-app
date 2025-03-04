import React, { forwardRef } from 'react';
import styled, { css } from 'styled-components';

import { Colors, Radius, Spacings } from '../../foundations';
import { Flex } from '../flex';
import { ButtonBase } from './ButtonBase';
import { ButtonProvider } from './ButtonContext';
import { ButtonIcon, ButtonText, StyledIcon, StyledText } from './components';

export interface ButtonProps extends React.ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: 'primary' | 'success' | 'destructive';
  size?: 'auto' | 'full' | '1/2';
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
  sizes: {
    auto: 'auto',
    full: '100%',
    '1/2': '50%',
  },
};

const StyledButton = styled(ButtonBase)<ButtonProps>`
  ${({ size: sizeProp = 'full', variant: variantProp = 'primary' }) => {
    const variant = styles.variants[variantProp];
    const size = styles.sizes[sizeProp];

    return css`
      --background: ${variant.background};
      --hover: ${variant.hover};
      --disabled: ${variant.disabled};
      --radius: ${styles.radius};
      --size: ${size};

      min-height: 32px;
      min-width: 60px;
      border-radius: var(--radius);
      width: var(--size);
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
    `;
  }}
`;

const StyledFlex = styled(Flex)`
  justify-content: space-between;
  &&:has(${StyledText}:only-child) {
    justify-content: center;
  }
  &&:has(${StyledText} + ${StyledIcon}) {
    &::before {
      content: ' ';
      display: inline-block;
      width: 24px;
    }
  }
  &&:has(${StyledIcon} + ${StyledText}) {
    &::after {
      content: ' ';
      display: inline-block;
      width: 24px;
    }
  }
  &&:has(${StyledIcon} + ${StyledText} + ${StyledIcon}) {
    &::before {
      display: none;
    }
    &::after {
      display: none;
    }
  }
`;

const Button = forwardRef<HTMLButtonElement, ButtonProps>(
  ({ children, disabled = false, style, ...props }, ref) => {
    return (
      <ButtonProvider disabled={disabled}>
        <StyledButton ref={ref} disabled={disabled} {...props}>
          <StyledFlex
            $flex={1}
            $gap={Spacings.small}
            $alignItems="center"
            $padding={{
              horizontal: Spacings.small,
            }}>
            {children}
          </StyledFlex>
        </StyledButton>
      </ButtonProvider>
    );
  },
);

Button.displayName = 'Button';

const ButtonNamespace = Object.assign(Button, {
  Text: ButtonText,
  Icon: ButtonIcon,
});

export { ButtonNamespace as Button };
