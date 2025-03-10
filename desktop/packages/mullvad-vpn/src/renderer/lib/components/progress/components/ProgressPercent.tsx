import styled from 'styled-components';

import { Colors } from '../../../foundations';
import { LabelTiny, LabelTinyProps } from '../../typography';
import { useProgress } from '../ProgressContext';

export type ProgressPercentProps<T extends React.ElementType> = LabelTinyProps<T>;

const StyledText = styled(LabelTiny)`
  min-width: 26px;
`;

export const ProgressPercent = <T extends React.ElementType>(props: ProgressPercentProps<T>) => {
  const { percent, disabled } = useProgress();
  return (
    <StyledText color={disabled ? Colors.white40 : Colors.white} {...props}>
      {`${Math.round(percent)}%`}
    </StyledText>
  );
};
