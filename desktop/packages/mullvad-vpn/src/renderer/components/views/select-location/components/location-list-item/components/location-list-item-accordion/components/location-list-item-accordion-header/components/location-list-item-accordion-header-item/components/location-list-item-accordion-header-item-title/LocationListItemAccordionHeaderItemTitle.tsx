import { useListItemContext } from '../../../../../../../../../../../../../lib/components/list-item/ListItemContext';
import {
  SelectableLabel,
  type SelectableLabelProps,
} from '../../../../../../../../../../../../../lib/components/selectable-label';
import { useLocationListItemContext } from '../../../../../../../../LocationListItemContext';

export type LocationListItemAccordionHeaderItemTitleProps<E extends React.ElementType = 'span'> =
  SelectableLabelProps<E>;

export const LocationListItemAccordionHeaderItemTitle = <E extends React.ElementType = 'span'>(
  props: LocationListItemAccordionHeaderItemTitleProps<E>,
) => {
  const { disabled } = useListItemContext();
  const { selected } = useLocationListItemContext();
  return <SelectableLabel selected={selected} disabled={disabled} {...props} />;
};
