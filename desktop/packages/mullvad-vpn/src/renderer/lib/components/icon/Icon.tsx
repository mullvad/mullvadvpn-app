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

const sizes = {
  tiny: 12,
  small: 16,
  medium: 24,
  large: 32,
  big: 48,
};

export const Icon = ({
  icon: iconProp,
  size = 'medium',
  color = Colors.white,
  ...props
}: IconProps) => {
  const icon = icons[iconProp];
  const src = iconProp.startsWith('data:') ? iconProp : `../../assets/icons/${icon}.svg`;
  return <StyledIcon $src={src} $size={sizes[size]} $color={color} role="img" {...props} />;
};
