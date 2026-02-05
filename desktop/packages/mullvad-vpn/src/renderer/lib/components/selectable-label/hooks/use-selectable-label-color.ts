import type { Colors } from '../../../foundations';

const colors: Record<string, Colors> = {
  default: 'white',
  disabled: 'whiteAlpha40',
  selected: 'green',
};

export function useSelectableLabelColor(selected?: boolean, disabled?: boolean) {
  let color = colors.default;
  if (disabled) {
    color = colors.disabled;
  } else if (selected) {
    color = colors.selected;
  }
  return color;
}
