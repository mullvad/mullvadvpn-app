import styled, { css } from 'styled-components';

import { colors } from '../../../foundations';
import { Icon, IconProps } from '../../icon/Icon';
import { IconButtonVariant } from '../IconButton';
import { useIconButtonContext } from '../IconButtonContext';
export type IconButtonIconProps = IconProps;

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

const StyledIconButtonIcon = styled(Icon)<{ $variant: IconButtonVariant }>(({ $variant }) => {
  const variant = variants[$variant];
  return css`
    --background: ${variant.background};
    --hover: ${variant.hover};
    --pressed: ${variant.pressed};
    --disabled: ${variant.disabled};
    --transition-duration: 0.15s;

    @media (prefers-reduced-motion: no-preference) {
      transition: background-color var(--transition-duration) ease;
    }

    background-color: var(--background);

    &&:not([data-disabled='true']):hover {
      --transition-duration: 0s;
      background-color: var(--hover);
    }

    &&:not([data-disabled='true']):active {
      --transition-duration: 0s;
      background-color: var(--pressed);
    }

    &&[data-disabled='true'] {
      background-color: var(--disabled);
    }
  `;
});

export const IconButtonIcon = (props: IconButtonIconProps) => {
  const { variant = 'primary', size, disabled } = useIconButtonContext();
  return (
    <StyledIconButtonIcon size={size} data-disabled={disabled} $variant={variant} {...props} />
  );
};
