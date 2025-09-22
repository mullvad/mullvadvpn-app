import { ListItemLabel, ListItemLabelProps } from '../../../list-item/components';
import { useListboxContext } from '../../';

export type ListboxLabelProps = ListItemLabelProps;

export const ListboxLabel = (props: ListboxLabelProps) => {
  const { labelId } = useListboxContext();
  return <ListItemLabel id={labelId} {...props} />;
};
