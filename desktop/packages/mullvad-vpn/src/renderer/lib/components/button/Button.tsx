import React, { forwardRef } from 'react';
import styled from 'styled-components';

import { Colors, Radius, Spacings } from '../../foundations';
import { buttonReset } from '../../styles';
import { Flex } from '../flex';
import { ButtonIcon, ButtonProvider, ButtonText, StyledIcon, StyledText } from './components';

export interface ButtonProps extends React.ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: 'primary' | 'success' | 'destructive';
  size?: 'auto' | 'full' | '1/2';
}

const variants = {
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
} as const;

const sizes = {
  auto: 'auto',
  full: '100%',
  '1/2': '50%',
};

const StyledButton = styled.button({
  ...buttonReset,

  minHeight: '32px',
  borderRadius: Radius.radius4,
  minWidth: '60px',
  width: 'var(--size)',
  background: 'var(--background)',
  '&:not(:disabled):hover': {
    background: 'var(--hover)',
  },
  '&:disabled': {
    background: 'var(--disabled)',
  },
  '&:focus-visible': {
    outline: `2px solid ${Colors.white}`,
    outlineOffset: '2px',
  },
});

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
  ({ variant = 'primary', size = 'full', children, disabled = false, style, ...props }, ref) => {
    const styles = variants[variant];
    return (
      <ButtonProvider disabled={disabled}>
        <StyledButton
          ref={ref}
          style={
            {
              '--background': styles.background,
              '--hover': styles.hover,
              '--disabled': styles.disabled,
              '--size': sizes[size],
              ...style,
            } as React.CSSProperties
          }
          disabled={disabled}
          {...props}>
          <StyledFlex
            $flex={1}
            $gap={Spacings.spacing3}
            $alignItems="center"
            $padding={{
              horizontal: Spacings.spacing3,
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
