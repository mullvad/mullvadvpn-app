import { type Colors } from '../../../../../foundations';
import { useFilterChipContext } from '../../../FilterChipContext';

export function useColor(): Colors {
  const { disabled, inactive } = useFilterChipContext();

  if (disabled) {
    return 'whiteAlpha40';
  }

  if (inactive) {
    return 'whiteAlpha20';
  }

  return 'white';
}
