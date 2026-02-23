import { useListItemContext } from '../../../../../lib/components/list-item/ListItemContext';
import type { Colors } from '../../../../../lib/foundations';
import { useLocationListItemContext } from '../../../LocationListItemContext';

const colors: Record<string, Colors> = {
  default: 'white',
  disabled: 'whiteAlpha40',
  selected: 'green',
};

export function useListItemLabelColor() {
  const { disabled } = useListItemContext();
  const { selected } = useLocationListItemContext();

  let color = colors.default;
  if (disabled) {
    color = colors.disabled;
  } else if (selected) {
    color = colors.selected;
  }
  return color;
}
