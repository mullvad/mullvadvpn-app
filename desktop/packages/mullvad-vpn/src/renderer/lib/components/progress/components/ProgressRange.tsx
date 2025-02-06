import styled from 'styled-components';

import { Colors, Radius } from '../../../foundations';
import { useProgress } from '../ProgressContext';

const StyledDiv = styled.div<{
  progress: number;
  disabled?: boolean;
}>`
  // TODO: Replace with token when available
  background-color: ${({ disabled }) => (disabled ? 'rgba(42, 97, 72, 1)' : Colors.green)};
  border-radius: ${Radius.radius4};
  height: 100%;
  width: 100%;
  transition: transform 0.3s ease-in-out;
  transform: translateX(${({ progress }) => progress - 100}%);
`;

export const ProgressRange = () => {
  const { percent, disabled } = useProgress();

  return <StyledDiv progress={percent} disabled={disabled} />;
};
