import { Colors } from '../../../../../../../foundations';
import { useListItemContext } from '../../../../../../list-item/ListItemContext';
import { useListboxOptionContext } from '../../../ListboxOptionContext';

const colors: Record<string, Colors> = {
  default: 'white',
  disabled: 'whiteAlpha40',
  selected: 'green',
};

export function useListboxOptionLabelColor() {
  const { disabled } = useListItemContext();
  const { selected } = useListboxOptionContext();

  let color = colors.default;
  if (disabled) {
    color = colors.disabled;
  } else if (selected) {
    color = colors.selected;
  }
  return color;
}
