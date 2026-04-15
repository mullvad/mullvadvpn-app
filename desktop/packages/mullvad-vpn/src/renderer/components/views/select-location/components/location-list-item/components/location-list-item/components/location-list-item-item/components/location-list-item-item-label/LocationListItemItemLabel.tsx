import { useListItemContext } from '../../../../../../../../../../../lib/components/list-item/ListItemContext';
import {
  SelectableLabel,
  type SelectableLabelProps,
} from '../../../../../../../../../../../lib/components/selectable-label';
import { useLocationContext } from '../../../../../../LocationContext';

export type LocationListItemItemLabelProps<E extends React.ElementType = 'span'> =
  SelectableLabelProps<E>;

export const LocationListItemItemLabel = <E extends React.ElementType = 'span'>(
  props: LocationListItemItemLabelProps<E>,
) => {
  const { disabled } = useListItemContext();
  const { selected } = useLocationContext();
  return <SelectableLabel selected={selected} disabled={disabled} {...props} />;
};
