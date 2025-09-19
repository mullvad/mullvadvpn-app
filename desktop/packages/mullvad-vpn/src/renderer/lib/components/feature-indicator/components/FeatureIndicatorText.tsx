import styled from 'styled-components';

import { BodySmallSemiBoldProps, LabelTiny } from '../../typography';
import { useFeatureIndicatorContext } from '../FeatureIndicatorContext';

export type FeatureIndicatorTextProps<T extends React.ElementType = 'span'> =
  BodySmallSemiBoldProps<T>;

export const StyledFeatureIndicatorText = styled(LabelTiny)``;

export const FeatureIndicatorText = <T extends React.ElementType = 'span'>(
  props: FeatureIndicatorTextProps<T>,
) => {
  const { disabled } = useFeatureIndicatorContext();
  return <StyledFeatureIndicatorText color={disabled ? 'whiteAlpha40' : 'white'} {...props} />;
};
