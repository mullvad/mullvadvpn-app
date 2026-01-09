import { Text, TextProps } from '../../../text';
import { useListItemContext } from '../../ListItemContext';

export type ListItemLabelProps<E extends React.ElementType = 'span'> = TextProps<E>;

export const ListItemLabel = <E extends React.ElementType = 'span'>(
  props: ListItemLabelProps<E>,
) => {
  const { disabled } = useListItemContext();
  return (
    <Text variant="bodySmallSemibold" color={disabled ? 'whiteAlpha40' : 'white'} {...props} />
  );
};
