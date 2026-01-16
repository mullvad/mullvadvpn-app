import { Colors } from '../../../../../foundations';
import { useCheckboxContext } from '../../../CheckboxContext';

export const useBorderColor = (): Colors => {
  const { disabled } = useCheckboxContext();
  if (disabled) return 'whiteAlpha20';
  return 'whiteAlpha80';
};
