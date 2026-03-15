import { ListItem } from '../../../list-item';
import type { ListItemItemLabelProps } from '../../../list-item/components/list-item-item/components';
import { useListboxContext } from '../../ListboxContext';

export type ListboxLabelProps = ListItemItemLabelProps;

export const ListboxLabel = (props: ListboxLabelProps) => {
  const { labelId } = useListboxContext();
  return <ListItem.Item.Label id={labelId} {...props} />;
};
