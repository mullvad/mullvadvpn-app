import { BodySmallSemiBoldProps, Text } from '../../text';
import { useFilterChipContext } from '../FilterChipContext';

export type FilterChipTextProps<T extends React.ElementType = 'span'> = BodySmallSemiBoldProps<T>;

export const FilterChipText = <T extends React.ElementType = 'span'>(
  props: FilterChipTextProps<T>,
) => {
  const { labelId, disabled } = useFilterChipContext();
  return (
    <Text
      id={labelId}
      variant="labelTinySemiBold"
      color={disabled ? 'whiteAlpha40' : 'white'}
      {...props}
    />
  );
};
