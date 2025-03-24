import { Colors } from '../../../foundations';
import { LabelTinyProps, TitleMedium } from '../../typography';
import { useListItem } from '../ListItemContext';

export type ListItemLabelProps<E extends React.ElementType = 'span'> = LabelTinyProps<E>;

export const ListItemLabel = <E extends React.ElementType = 'span'>(
  props: ListItemLabelProps<E>,
) => {
  const { disabled } = useListItem();
  return <TitleMedium color={disabled ? Colors.white40 : Colors.white} {...props} />;
};
