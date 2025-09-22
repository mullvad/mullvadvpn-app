import { Colors } from '../../../../../foundations';
import { useSwitchContext } from '../../../SwitchContext';

export const useBorderColor = (): Colors => {
  const { disabled } = useSwitchContext();
  if (disabled) return 'whiteAlpha20';
  return 'whiteAlpha80';
};
