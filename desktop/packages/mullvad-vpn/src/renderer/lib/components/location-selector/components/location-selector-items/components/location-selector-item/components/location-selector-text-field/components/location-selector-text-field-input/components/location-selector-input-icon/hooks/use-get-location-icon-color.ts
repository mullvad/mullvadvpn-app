import type { colors } from '../../../../../../../../../../../../../foundations';
import { useTextFieldContext } from '../../../../../../../../../../../../text-field';

export function useGetLocationIconColor(selected: boolean): keyof typeof colors {
  const { invalid } = useTextFieldContext();
  if (invalid) {
    return 'red';
  }
  return selected ? 'white' : 'whiteAlpha60';
}
