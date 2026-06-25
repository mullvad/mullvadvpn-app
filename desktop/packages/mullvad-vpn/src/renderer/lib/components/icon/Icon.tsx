import styled, { css } from 'styled-components';

import { Colors, colors } from '../../foundations';
import { icons } from './types';

export type IconProps = {
  icon: keyof typeof icons | 'placeholder';
  size?: 'tiny' | 'small' | 'medium' | 'large' | 'big';
  color?: Colors;
  className?: string;
} & React.HTMLAttributes<HTMLDivElement>;

export const StyledPlaceholder = styled.div<{ $size: number }>`
  ${({ $size }) => {
    return css`
      flex-shrink: 0;
      width: ${$size}px;
      height: ${$size}px;
    `;
  }}
`;

export const StyledIcon = styled.div<{ $color: string; $size: number; $src: string }>`
  ${({ $size, $src, $color }) => {
    return css`
      flex-shrink: 0;
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

export const Icon = ({
  icon: iconProp,
  size: sizeProp = 'medium',
  color: colorProp = 'white',
  ...props
}: IconProps) => {
  const size = iconSizes[sizeProp];

  if (iconProp === 'placeholder') {
    return <StyledPlaceholder $size={size} />;
  }

  const icon = icons[iconProp];
  const src = iconProp.startsWith('data:') ? iconProp : `assets/icons/${icon}.svg`;

  const color = colors[colorProp];
  return <StyledIcon $src={src} $size={size} $color={color} role="img" {...props} />;
};
