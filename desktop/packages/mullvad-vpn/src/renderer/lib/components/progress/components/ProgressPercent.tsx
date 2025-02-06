import styled from 'styled-components';

import { Colors } from '../../../foundations';
import { LabelTiny, LabelTinyProps } from '../../typography';
import { useProgress } from '../ProgressContext';

export type ProgressPercentProps = Omit<LabelTinyProps, 'children'>;

const StyledText = styled(LabelTiny)`
  min-width: 26px;
`;

export const ProgressPercent = (props: ProgressPercentProps) => {
  const { percent, disabled } = useProgress();
  return (
    <StyledText color={disabled ? Colors.white40 : Colors.white} {...props}>
      {`${Math.round(percent)}%`}
    </StyledText>
  );
};
