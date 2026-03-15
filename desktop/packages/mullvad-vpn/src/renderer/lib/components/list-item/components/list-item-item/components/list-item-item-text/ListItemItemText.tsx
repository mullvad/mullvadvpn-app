import { Text, TextProps } from '../../../../../text';
import { useListItemContext } from '../../../../ListItemContext';

export type ListItemItemTextProps<E extends React.ElementType = 'span'> = TextProps<E>;

export const ListItemItemText = <E extends React.ElementType = 'span'>(
  props: ListItemItemTextProps<E>,
) => {
  const { disabled } = useListItemContext();
  return <Text variant="labelTiny" color={disabled ? 'whiteAlpha40' : 'whiteAlpha60'} {...props} />;
};
