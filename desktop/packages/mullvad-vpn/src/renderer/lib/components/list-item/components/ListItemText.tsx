import { Text, TextProps } from '../../typography';
import { useListItem } from '../ListItemContext';

export type ListItemTextProps<E extends React.ElementType = 'span'> = TextProps<E>;

export const ListItemText = <E extends React.ElementType = 'span'>(props: ListItemTextProps<E>) => {
  const { disabled } = useListItem();
  return <Text variant="labelTiny" color={disabled ? 'whiteAlpha40' : 'whiteAlpha60'} {...props} />;
};
