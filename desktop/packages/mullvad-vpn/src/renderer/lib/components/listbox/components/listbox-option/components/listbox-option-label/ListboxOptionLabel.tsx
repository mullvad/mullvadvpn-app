import { TextProps } from '../../../../..';
import { SelectableLabel } from '../../../../../selectable-label';
import { useListboxOptionContext } from '../../ListboxOptionContext';
import { useListboxOptionLabelColor } from './hooks';

export type SelectListItemOptionLabelProps<E extends React.ElementType = 'span'> = TextProps<E>;

export const ListboxOptionLabel = <E extends React.ElementType = 'span'>(
  props: SelectListItemOptionLabelProps<E>,
) => {
  const color = useListboxOptionLabelColor();
  const { selected } = useListboxOptionContext();
  return <SelectableLabel selected={selected} color={color} {...props} />;
};
