import { useListItemContext } from '../../../../../../../../../../../../../lib/components/list-item/ListItemContext';
import {
  SelectableLabel,
  type SelectableLabelProps,
} from '../../../../../../../../../../../../../lib/components/selectable-label';
import { useLocationContext } from '../../../../../../../../LocationContext';

export type LocationAccordionHeaderItemTitleProps<E extends React.ElementType = 'span'> =
  SelectableLabelProps<E>;

export const LocationAccordionHeaderItemTitle = <E extends React.ElementType = 'span'>(
  props: LocationAccordionHeaderItemTitleProps<E>,
) => {
  const { disabled } = useListItemContext();
  const { selected } = useLocationContext();
  return <SelectableLabel selected={selected} disabled={disabled} {...props} />;
};
