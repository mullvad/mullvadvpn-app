import React from 'react';
import styled, { css } from 'styled-components';

import { colors } from '../../foundations';
import { IconProps, iconSizes } from '../icon/Icon';
import { IconButtonIcon, StyledIconButtonIcon } from './components';
import { IconButtonProvider } from './IconButtonContext';

export type IconButtonVariant = 'primary' | 'secondary';

export type IconButtonProps = React.ComponentPropsWithRef<'button'> & {
  variant?: IconButtonVariant;
  size?: IconProps['size'];
};

const variants: Record<
  IconButtonVariant,
  {
    background: string;
    hover: string;
    pressed: string;
    disabled: string;
  }
> = {
  primary: {
    background: colors.white,
    hover: colors.whiteAlpha60,
    pressed: colors.whiteAlpha40,
    disabled: colors.whiteAlpha40,
  },
  secondary: {
    background: colors.whiteAlpha60,
    hover: colors.whiteAlpha80,
    pressed: colors.white,
    disabled: colors.whiteAlpha40,
  },
} as const;

const StyledButton = styled.button<{
  $size: IconButtonProps['size'];
  $variant: IconButtonVariant;
}>`
  ${({ $size = 'medium', $variant = 'primary' }) => {
    const size = iconSizes[$size];
    const variant = variants[$variant];
    return css`
      --size: ${size}px;

      --background: ${variant.background};
      --hover: ${variant.hover};
      --pressed: ${variant.pressed};
      --disabled: ${variant.disabled};
      --transition-duration: 0.15s;

      background: ${colors.transparent};
      height: var(--size);
      width: var(--size);
      border-radius: 100%;

      &&:focus-visible {
        outline: 2px solid ${colors.white};
        outline-offset: 1px;
      }

      ${StyledIconButtonIcon} {
        background-color: var(--background);
        @media (prefers-reduced-motion: no-preference) {
          transition: background-color var(--transition-duration) ease;
        }
      }

      &&:not(:disabled):hover ${StyledIconButtonIcon} {
        --transition-duration: 0s;
        background-color: var(--hover);
      }

      &&:not(:disabled):active ${StyledIconButtonIcon} {
        --transition-duration: 0s;
        background-color: var(--pressed);
      }

      &&:disabled ${StyledIconButtonIcon} {
        background-color: var(--disabled);
      }
    `;
  }}
`;

function IconButton({ variant = 'primary', size = 'medium', disabled, ...props }: IconButtonProps) {
  return (
    <IconButtonProvider size={size} variant={variant} disabled={disabled}>
      <StyledButton disabled={disabled} $variant={variant} $size={size} {...props} />
    </IconButtonProvider>
  );
}

const IconButtonNamespace = Object.assign(IconButton, {
  Icon: IconButtonIcon,
});

export { IconButtonNamespace as IconButton };
