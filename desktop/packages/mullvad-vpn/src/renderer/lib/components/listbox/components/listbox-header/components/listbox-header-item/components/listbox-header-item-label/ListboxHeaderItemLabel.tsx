import { ListItem } from '../../../../../../../list-item';
import type { ListItemItemLabelProps } from '../../../../../../../list-item/components/list-item-item/components';
import { useListboxContext } from '../../../../../../ListboxContext';

export type ListboxHeaderItemLabelProps = ListItemItemLabelProps;

export const ListboxHeaderItemLabel = (props: ListboxHeaderItemLabelProps) => {
  const { labelId } = useListboxContext();
  return <ListItem.Item.Label id={labelId} {...props} />;
};
