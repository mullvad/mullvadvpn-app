import styled from 'styled-components';

import { Colors } from '../../../foundations';
import { BodySmallSemiBoldProps, LabelTiny } from '../../typography';
import { useFilterChipContext } from '../FilterChipContext';

export type FilterChipTextProps<T extends React.ElementType> = BodySmallSemiBoldProps<T>;

export const StyledText = styled(LabelTiny)``;

export const FilterChipText = <T extends React.ElementType>(props: FilterChipTextProps<T>) => {
  const { disabled } = useFilterChipContext();
  return <StyledText color={disabled ? Colors.white40 : Colors.white} {...props} />;
};
