import styled from 'styled-components';

import { Colors } from '../../foundations';

export interface NotificationProps {
  variant?: 'success' | 'warning' | 'error';
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

const colors = {
  success: Colors.green,
  warning: Colors.yellow,
  error: Colors.red,
};

export const Notification = ({
  variant = 'success',
  size = 'medium',
  ...props
}: NotificationProps) => {
  return <StyledDiv $size={sizes[size]} $color={colors[variant]} {...props} />;
};
