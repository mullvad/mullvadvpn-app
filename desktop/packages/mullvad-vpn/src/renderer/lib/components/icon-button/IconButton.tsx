import React, { forwardRef } from 'react';
import styled from 'styled-components';

import { Colors } from '../../foundations';
import { buttonReset } from '../../styles';
import { IconProps, iconSizes } from '../icon/Icon';
import { IconButtonIcon } from './components/IconButtonIcon';
import { IconButtonProvider } from './IconButtonContext';

export interface IconButtonProps extends React.ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: 'primary' | 'secondary';
  size?: IconProps['size'];
}

const StyledButton = styled.button({
  ...buttonReset,

  background: 'transparent',
  height: 'var(--size)',
  width: 'var(--size)',
  borderRadius: '100%',
  '&:focus-visible': {
    outline: `2px solid ${Colors.white}`,
    outlineOffset: '1px',
  },
});

const IconButton = forwardRef<HTMLButtonElement, IconButtonProps>(
  ({ variant = 'primary', size: sizeProp = 'medium', disabled, style, ...props }, ref) => {
    const size = iconSizes[sizeProp];
    return (
      <IconButtonProvider size={sizeProp} variant={variant} disabled={disabled}>
        <StyledButton
          ref={ref}
          disabled={disabled}
          style={
            {
              '--size': `${size}px`,
              ...style,
            } as React.CSSProperties
          }
          {...props}
        />
      </IconButtonProvider>
    );
  },
);

const IconButtonNamespace = Object.assign(IconButton, {
  Icon: IconButtonIcon,
});

export { IconButtonNamespace as IconButton };

IconButton.displayName = 'Button';
