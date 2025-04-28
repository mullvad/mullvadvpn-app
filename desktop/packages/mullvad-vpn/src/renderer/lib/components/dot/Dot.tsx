import styled from 'styled-components';

import { colors } from '../../foundations';

export interface DotProps {
  variant?: 'primary' | 'success' | 'warning' | 'error';
  size?: 'tiny' | 'small' | 'medium';
}

const StyledDiv = styled.div<{ $size: string; $color: string }>`
  width: ${({ $size }) => $size};
  height: ${({ $size }) => $size};
  border-radius: 50%;
  background-color: ${({ $color }) => $color};
`;

const sizes = {
  tiny: '8px',
  small: '10px',
  medium: '12px',
};

const dotColors = {
  primary: colors.white80,
  success: colors.brandGreen,
  warning: colors.brandYellow,
  error: colors.brandRed,
};

export const Dot = ({ variant = 'primary', size = 'medium', ...props }: DotProps) => {
  return <StyledDiv $size={sizes[size]} $color={dotColors[variant]} {...props} />;
};
