import React, { forwardRef } from 'react';
import styled, { css } from 'styled-components';

import { colors } from '../../foundations';
import { IconProps, iconSizes } from '../icon/Icon';
import { IconButtonIcon } from './components/IconButtonIcon';
import { IconButtonProvider } from './IconButtonContext';

export interface IconButtonProps extends React.ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: 'primary' | 'secondary';
  size?: IconProps['size'];
}

const StyledButton = styled.button<{ $size: IconButtonProps['size'] }>`
  ${({ $size = 'medium' }) => {
    const size = iconSizes[$size];
    return css`
      --size: ${size}px;

      background: ${colors.transparent};
      height: var(--size);
      width: var(--size);
      border-radius: 100%;
      &:focus-visible {
        outline: 2px solid ${colors.white};
        outline-offset: 1px;
      }
    `;
  }}
`;

const IconButton = forwardRef<HTMLButtonElement, IconButtonProps>(
  ({ variant = 'primary', size = 'medium', disabled, style, ...props }, ref) => {
    return (
      <IconButtonProvider size={size} variant={variant} disabled={disabled}>
        <StyledButton ref={ref} disabled={disabled} $size={size} {...props} />
      </IconButtonProvider>
    );
  },
);

const IconButtonNamespace = Object.assign(IconButton, {
  Icon: IconButtonIcon,
});

export { IconButtonNamespace as IconButton };

IconButton.displayName = 'Button';
