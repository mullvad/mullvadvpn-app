import React, { forwardRef } from 'react';
import styled from 'styled-components';

import { Colors } from '../../foundations';
import { buttonReset } from '../../styles';
import { Icon, IconProps } from '../icon/Icon';

export interface IconButtonProps
  extends Omit<React.ButtonHTMLAttributes<HTMLButtonElement>, 'children'> {
  variant?: 'primary' | 'secondary';
  size?: IconProps['size'];
  icon: string;
}

const variants = {
  primary: {
    background: Colors.white,
    hover: Colors.white60,
    disabled: Colors.white50,
  },
  secondary: {
    background: Colors.white60,
    hover: Colors.white80,
    disabled: Colors.white50,
  },
} as const;

const sizes = {
  small: 12,
  medium: 16,
  large: 24,
  big: 32,
  huge: 48,
};

const StyledButton = styled.button({
  ...buttonReset,

  background: 'transparent',
  height: 'var(--size)',
  width: 'var(--size)',
  borderRadius: '100%',
  '&:focus-visible': {
    outline: `2px solid ${Colors.white}`,
  },
});

const StyledIcon = styled(Icon)<IconProps & { $hoverColor: string; $disabled?: boolean }>(
  ({ $hoverColor, $disabled }) => ({
    ...(!$disabled && {
      '&&:hover': {
        backgroundColor: $hoverColor,
      },
    }),
  }),
);

export const IconButton = forwardRef<HTMLButtonElement, IconButtonProps>(
  ({ icon, variant = 'primary', size: sizeProp = 'large', disabled, style, ...props }, ref) => {
    const styles = variants[variant];
    const size = sizes[sizeProp];
    return (
      <StyledButton
        ref={ref}
        disabled={disabled}
        style={
          {
            '--size': `${size}px`,
            ...style,
          } as React.CSSProperties
        }
        {...props}>
        <StyledIcon
          src={icon}
          color={styles.background}
          size={sizeProp}
          $hoverColor={styles.hover}
          $disabled={disabled}
        />
      </StyledButton>
    );
  },
);

IconButton.displayName = 'Button';
