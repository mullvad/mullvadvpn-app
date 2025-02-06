import styled from 'styled-components';

import { Colors, Radius } from '../../../foundations';
import { useProgress } from '../ProgressContext';

const StyledDiv = styled.div<{
  disabled?: boolean;
}>`
  // TODO: Replace with token when available
  background-color: ${({ disabled }) => (disabled ? 'rgba(42, 97, 72, 1)' : Colors.green)};
  border-radius: ${Radius.radius4};
  height: 100%;
  width: 100%;
  transition: transform 0.3s ease-in-out;
  transform: var(--transform);
`;

export const ProgressRange = () => {
  const { percent, disabled } = useProgress();
  const transform = `translateX(${percent - 100}%)`;

  return (
    <StyledDiv disabled={disabled} style={{ '--transform': transform } as React.CSSProperties} />
  );
};
