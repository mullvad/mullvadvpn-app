import styled from 'styled-components';

import { BodySmallSemiBoldProps, LabelTinySemiBold } from '../../text';
import { useFilterChipContext } from '../FilterChipContext';

export type FilterChipTextProps<T extends React.ElementType = 'span'> = BodySmallSemiBoldProps<T>;

export const StyledText = styled(LabelTinySemiBold)``;

export const FilterChipText = <T extends React.ElementType = 'span'>(
  props: FilterChipTextProps<T>,
) => {
  const { disabled } = useFilterChipContext();
  return <StyledText color={disabled ? 'whiteAlpha40' : 'white'} {...props} />;
};
