import styled from 'styled-components';

import { BodySmallSemiBoldProps, LabelTinySemiBold } from '../../../text';
import { useFeatureIndicatorContext } from '../../FeatureIndicatorContext';

export type FeatureIndicatorTextProps<T extends React.ElementType = 'span'> =
  BodySmallSemiBoldProps<T>;

export const StyledFeatureIndicatorText = styled(LabelTinySemiBold)``;

export const FeatureIndicatorText = <T extends React.ElementType = 'span'>(
  props: FeatureIndicatorTextProps<T>,
) => {
  const { disabled, inactive } = useFeatureIndicatorContext();
  const color = disabled || inactive ? 'whiteAlpha20' : 'white';

  return <StyledFeatureIndicatorText color={color} {...props} />;
};
