import { Colors } from '../../../../../foundations';
import { useSwitchContext } from '../../../SwitchContext';

export const useBackgroundColor = (): Colors => {
  const { disabled, checked } = useSwitchContext();
  if (disabled) {
    if (checked) return 'green40';
    else return 'red40';
  }
  if (checked) return 'green';
  else return 'red';
};
