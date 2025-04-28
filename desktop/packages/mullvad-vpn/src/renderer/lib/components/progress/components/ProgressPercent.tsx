import styled from 'styled-components';

import { DeprecatedColors } from '../../../foundations';
import { LabelTiny, LabelTinyProps } from '../../typography';
import { useProgress } from '../ProgressContext';

export type ProgressPercentProps<T extends React.ElementType = 'span'> = LabelTinyProps<T>;

const StyledText = styled(LabelTiny)`
  min-width: 26px;
`;

export const ProgressPercent = <T extends React.ElementType = 'span'>(
  props: ProgressPercentProps<T>,
) => {
  const { percent, disabled } = useProgress();
  return (
    <StyledText color={disabled ? DeprecatedColors.white40 : DeprecatedColors.white} {...props}>
      {`${Math.round(percent)}%`}
    </StyledText>
  );
};
