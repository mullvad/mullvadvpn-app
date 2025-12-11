import { Text, TextProps } from '../../../../..';
import { useListboxOptionLabelColor } from './hooks';

export type SelectListItemOptionLabelProps<E extends React.ElementType = 'span'> = TextProps<E>;

export const ListboxOptionLabel = <E extends React.ElementType = 'span'>(
  props: SelectListItemOptionLabelProps<E>,
) => {
  const color = useListboxOptionLabelColor();
  return <Text variant="bodySmallSemibold" color={color} {...props} />;
};
