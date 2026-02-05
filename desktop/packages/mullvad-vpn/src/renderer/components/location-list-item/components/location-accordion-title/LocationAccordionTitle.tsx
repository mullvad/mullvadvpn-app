import { type TextProps } from '../../../../lib/components';
import { useListItemContext } from '../../../../lib/components/list-item/ListItemContext';
import { SelectableLabel } from '../../../../lib/components/selectable-label';
import { useLocationListItemContext } from '../../LocationListItemContext';

export type LocationAccordionTitleProps<E extends React.ElementType = 'span'> = TextProps<E>;

export const LocationAccordionTitle = <E extends React.ElementType = 'span'>(
  props: LocationAccordionTitleProps<E>,
) => {
  const { disabled } = useListItemContext();
  const { selected } = useLocationListItemContext();
  return <SelectableLabel selected={selected} disabled={disabled} {...props} />;
};
