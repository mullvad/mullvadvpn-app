import { Text, TextProps } from '../../../../../text';
import { useListItemContext } from '../../../../ListItemContext';

export type ListItemItemLabelProps<E extends React.ElementType = 'span'> = TextProps<E>;

export const ListItemItemLabel = <E extends React.ElementType = 'span'>(
  props: ListItemItemLabelProps<E>,
) => {
  const { disabled } = useListItemContext();
  return (
    <Text variant="bodySmallSemibold" color={disabled ? 'whiteAlpha40' : 'white'} {...props} />
  );
};
