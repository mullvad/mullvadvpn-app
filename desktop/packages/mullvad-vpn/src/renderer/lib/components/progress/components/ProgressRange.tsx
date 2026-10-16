import styled from 'styled-components';

import { colors, durations, Radius } from '../../../foundations';
import { useProgress } from '../ProgressContext';

const StyledDiv = styled.div<{
  disabled?: boolean;
}>`
  background-color: ${({ disabled }) => (disabled ? colors.whiteAlpha80 : colors.white)};
  border-radius: ${Radius.radius4};
  height: 100%;
  width: 100%;
  transition: transform ${durations.fast} ease-out;
  transform: var(--transform);
`;

export const ProgressRange = () => {
  const { percent, disabled } = useProgress();
  const transform = `translateX(${percent - 100}%)`;

  return (
    <StyledDiv disabled={disabled} style={{ '--transform': transform } as React.CSSProperties} />
  );
};
