import { BodySmallSemiBoldProps, FootnoteMiniSemiBold } from '../../text';
import { useFilterChipContext } from '../FilterChipContext';

export type FilterChipTextProps<T extends React.ElementType = 'span'> = BodySmallSemiBoldProps<T>;

export const FilterChipText = <T extends React.ElementType = 'span'>(
  props: FilterChipTextProps<T>,
) => {
  const { disabled } = useFilterChipContext();
  return <FootnoteMiniSemiBold color={disabled ? 'whiteAlpha40' : 'white'} {...props} />;
};
