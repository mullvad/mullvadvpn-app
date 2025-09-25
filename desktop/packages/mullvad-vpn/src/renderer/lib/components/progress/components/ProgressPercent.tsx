import styled from 'styled-components';

import { LabelTinySemiBold, LabelTinySemiBoldProps } from '../../text';
import { useProgress } from '../ProgressContext';

export type ProgressPercentProps<T extends React.ElementType = 'span'> = LabelTinySemiBoldProps<T>;

const StyledText = styled(LabelTinySemiBold)`
  min-width: 26px;
`;

export const ProgressPercent = <T extends React.ElementType = 'span'>({
  color,
  ...props
}: ProgressPercentProps<T>) => {
  const { percent, disabled } = useProgress();
  const defaultColor = disabled ? 'whiteAlpha40' : 'white';
  return (
    <StyledText color={color ?? defaultColor} {...props}>
      {`${Math.round(percent)}%`}
    </StyledText>
  );
};
