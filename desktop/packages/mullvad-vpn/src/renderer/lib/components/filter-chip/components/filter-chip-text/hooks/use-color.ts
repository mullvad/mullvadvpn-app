import { type Colors } from '../../../../../foundations';
import { useFilterChipContext } from '../../../FilterChipContext';

export function useColor(): Colors {
  const { disabled, inactive } = useFilterChipContext();

  if (disabled || inactive) {
    return 'whiteAlpha40';
  }

  return 'white';
}
