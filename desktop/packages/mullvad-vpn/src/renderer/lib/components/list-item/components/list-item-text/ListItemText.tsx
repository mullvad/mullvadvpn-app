import { Text, TextProps } from '../../../typography';
import { useListItemContext } from '../../ListItemContext';

export type ListItemTextProps<E extends React.ElementType = 'span'> = TextProps<E>;

export const ListItemText = <E extends React.ElementType = 'span'>(props: ListItemTextProps<E>) => {
  const { disabled } = useListItemContext();
  return (
    <Text
      variant="labelTinySemiBold"
      color={disabled ? 'whiteAlpha40' : 'whiteAlpha60'}
      {...props}
    />
  );
};
