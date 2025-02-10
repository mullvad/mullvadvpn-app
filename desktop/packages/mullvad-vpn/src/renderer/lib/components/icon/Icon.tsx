import styled from 'styled-components';

import { Colors } from '../../foundations';
import { TransientProps } from '../../types';
import { icons } from './types';

export type IconProps = {
  icon: keyof typeof icons;
  size?: 'tiny' | 'small' | 'medium' | 'large' | 'big';
  color?: Colors;
  className?: string;
} & React.HTMLAttributes<HTMLDivElement>;

const StyledIcon = styled.div<
  TransientProps<Pick<IconProps, 'color'>> & { $size: number; $src: string }
>`
  width: ${({ $size }) => $size}px;
  height: ${({ $size }) => $size}px;
  mask: url(${({ $src }) => $src}) no-repeat center;
  mask-size: contain;
  background-color: ${({ $color }) => $color || 'currentColor'};
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
  color = Colors.white,
  ...props
}: IconProps) => {
  const icon = icons[iconProp];
  const src = iconProp.startsWith('data:') ? iconProp : `${PATH_PREFIX}/assets/icons/${icon}.svg`;
  return <StyledIcon $src={src} $size={iconSizes[size]} $color={color} role="img" {...props} />;
};
