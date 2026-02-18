import { useListItemContext } from '../../../../lib/components/list-item/ListItemContext';
import {
  SelectableLabel,
  type SelectableLabelProps,
} from '../../../../lib/components/selectable-label';
import { useLocationListItemContext } from '../../LocationListItemContext';

export type LocationListItemAccordionHeaderTitleProps<E extends React.ElementType = 'span'> =
  SelectableLabelProps<E>;

export const LocationListItemAccordionHeaderTitle = <E extends React.ElementType = 'span'>(
  props: LocationListItemAccordionHeaderTitleProps<E>,
) => {
  const { disabled } = useListItemContext();
  const { selected } = useLocationListItemContext();
  return <SelectableLabel selected={selected} disabled={disabled} {...props} />;
};
