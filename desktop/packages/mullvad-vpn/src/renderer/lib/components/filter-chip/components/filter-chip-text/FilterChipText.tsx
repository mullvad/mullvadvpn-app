import { BodySmallSemiBoldProps, FootnoteMiniSemiBold } from '../../../text';
import { useColor } from './hooks';

export type FilterChipTextProps<T extends React.ElementType = 'span'> = BodySmallSemiBoldProps<T>;

export const FilterChipText = <T extends React.ElementType = 'span'>(
  props: FilterChipTextProps<T>,
) => {
  const color = useColor();

  return <FootnoteMiniSemiBold color={color} {...props} />;
};
