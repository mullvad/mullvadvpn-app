import styled from 'styled-components';

import { TransientProps } from '../../types';

export type IconProps = {
  src: string;
  size?: 'small' | 'medium' | 'large' | 'big' | 'huge';
  color?: string;
  alt?: string;
  className?: string;
};

const StyledIcon = styled.div<TransientProps<Pick<IconProps, 'color' | 'src'>> & { $size: number }>`
  width: ${({ $size }) => $size}px;
  height: ${({ $size }) => $size}px;
  mask: url(${({ $src }) => $src}) no-repeat center;
  mask-size: contain;
  background-color: ${({ $color }) => $color || 'currentColor'};
`;

const sizes = {
  small: 12,
  medium: 16,
  large: 24,
  big: 32,
  huge: 48,
};

export const Icon = ({ src: srcProp, size = 'large', color, alt, ...props }: IconProps) => {
  const src = srcProp.startsWith('data:') ? srcProp : `../../assets/images/${srcProp}.svg`;
  return (
    <StyledIcon
      $src={src}
      $size={sizes[size]}
      $color={color}
      role="img"
      aria-label={alt || srcProp}
      {...props}
    />
  );
};
