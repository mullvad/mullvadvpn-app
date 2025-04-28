import { DeprecatedColors } from '../../../foundations';
import { Text, TextProps } from '../../typography';
import { useListItem } from '../ListItemContext';

export type ListItemProps<E extends React.ElementType = 'span'> = TextProps<E>;

export const ListItemText = <E extends React.ElementType = 'span'>(props: ListItemProps<E>) => {
  const { disabled } = useListItem();
  return (
    <Text
      variant="labelTiny"
      color={disabled ? DeprecatedColors.white40 : DeprecatedColors.white60}
      {...props}
    />
  );
};
