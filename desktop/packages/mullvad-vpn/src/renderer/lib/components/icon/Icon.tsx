import styled, { css } from 'styled-components';

import { Colors, colors } from '../../foundations';
import { icons } from './types';

export type IconProps = {
  icon: keyof typeof icons;
  size?: 'tiny' | 'small' | 'medium' | 'large' | 'big';
  color?: Colors;
  className?: string;
} & React.HTMLAttributes<HTMLDivElement>;

const StyledIcon = styled.div<{ $color: string; $size: number; $src: string }>`
  ${({ $size, $src, $color }) => {
    return css`
      width: ${$size}px;
      height: ${$size}px;
      mask: url(${$src}) no-repeat center;
      mask-size: contain;
      background-color: ${$color};
    `;
  }}
`;

export const iconSizes = {
  tiny: 14,
  small: 18,
  medium: 24,
  large: 32,
  big: 48,
};

const PATH_PREFIX = process.env.NODE_ENV === 'development' ? '../' : '';

export const Icon = ({
  icon: iconProp,
  size = 'medium',
  color: colorProp = 'white100',
  ...props
}: IconProps) => {
  const icon = icons[iconProp];
  const src = iconProp.startsWith('data:') ? iconProp : `${PATH_PREFIX}assets/icons/${icon}.svg`;
  const color = colors[colorProp];
  return <StyledIcon $src={src} $size={iconSizes[size]} $color={color} role="img" {...props} />;
};
