import styled from 'styled-components';

import { LabelTiny, LabelTinyProps } from '../../typography';
import { useProgress } from '../ProgressContext';

export type ProgressPercentProps<T extends React.ElementType = 'span'> = LabelTinyProps<T>;

const StyledText = styled(LabelTiny)`
  min-width: 26px;
`;

export const ProgressPercent = <T extends React.ElementType = 'span'>({
  color,
  ...props
}: ProgressPercentProps<T>) => {
  const { percent, disabled } = useProgress();
  const defaultColor = disabled ? 'white40' : 'white100';
  return (
    <StyledText color={color ?? defaultColor} {...props}>
      {`${Math.round(percent)}%`}
    </StyledText>
  );
};
