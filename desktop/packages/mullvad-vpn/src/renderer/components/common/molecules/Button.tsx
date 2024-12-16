import React, { forwardRef } from 'react';
import styled from 'styled-components';

import { Flex } from '../layout';
import { buttonReset } from '../mixins';
import { BodySmallSemiBold } from '../text';
import { Colors, Radius, Spacings } from '../variables';

export interface ButtonProps extends React.ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: 'primary' | 'success' | 'destructive';
  size?: 'tiny' | 'small' | 'regular' | 'full' | '1/2';
  leading?: React.ReactNode;
  trailing?: React.ReactNode;
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
  tiny: '44px',
  small: '60px',
  regular: '272px',
  full: '100%',
  '1/2': '50%',
};

const StyledButton = styled.button({
  ...buttonReset,

  minHeight: '32px',
  borderRadius: Radius.radius4,
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

export const Button = forwardRef<HTMLButtonElement, ButtonProps>(
  (
    { variant = 'primary', size = 'regular', leading, trailing, children, disabled, ...props },
    ref,
  ) => {
    const styles = variants[variant];
    return (
      <StyledButton
        ref={ref}
        style={
          {
            '--background': styles.background,
            '--hover': styles.hover,
            '--disabled': styles.disabled,
            '--size': sizes[size],
          } as React.CSSProperties
        }
        disabled={disabled}
        {...props}>
        <Flex
          $flex={1}
          $gap={Spacings.spacing3}
          $justifyContent="space-between"
          $padding={{
            horizontal: Spacings.spacing3,
          }}
          $alignItems="center">
          {leading}
          <Flex $flex={1} $justifyContent="center" $alignItems="center">
            {typeof children === 'string' ? (
              <BodySmallSemiBold color={disabled ? Colors.white40 : Colors.white}>
                {children}
              </BodySmallSemiBold>
            ) : (
              children
            )}
          </Flex>
          {trailing}
        </Flex>
      </StyledButton>
    );
  },
);

Button.displayName = 'Button';
