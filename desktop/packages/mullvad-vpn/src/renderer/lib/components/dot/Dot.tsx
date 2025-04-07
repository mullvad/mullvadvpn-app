import styled from 'styled-components';

import { Colors } from '../../foundations';

export interface DotProps {
  variant?: 'primary' | 'success' | 'warning' | 'error';
  size?: 'tiny' | 'small' | 'medium';
}

const StyledDiv = styled.div<{ $size: string; $color: string }>`
  min-width: ${({ $size }) => $size};
  min-height: ${({ $size }) => $size};
  border-radius: 50%;
  background-color: ${({ $color }) => $color};
`;

const sizes = {
  tiny: '8px',
  small: '10px',
  medium: '12px',
};

const colors = {
  primary: Colors.white80,
  success: Colors.green,
  warning: Colors.yellow,
  error: Colors.red,
};

export const Dot = ({ variant = 'primary', size = 'medium', ...props }: DotProps) => {
  return <StyledDiv $size={sizes[size]} $color={colors[variant]} {...props} />;
};
