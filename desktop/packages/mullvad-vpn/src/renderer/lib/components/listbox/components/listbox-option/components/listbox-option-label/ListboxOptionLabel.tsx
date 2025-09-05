import { Text, TextProps } from '../../../../..';
import { useListItemContext } from '../../../../../list-item/ListItemContext';

export type SelectListItemOptionLabelProps<E extends React.ElementType = 'span'> = TextProps<E>;

export const ListboxOptionLabel = <E extends React.ElementType = 'span'>(
  props: SelectListItemOptionLabelProps<E>,
) => {
  const { disabled } = useListItemContext();
  return <Text variant="bodySmall" color={disabled ? 'whiteAlpha40' : 'white'} {...props} />;
};
