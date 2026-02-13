import { Colors } from '../../../../../foundations';
import { useCheckboxContext } from '../../../CheckboxContext';

export const useBackgroundColor = (): Colors => {
  const { disabled, checked } = useCheckboxContext();
  if (disabled) {
    if (checked) return 'green40';
    else return 'red40';
  }
  if (checked) return 'green';
  else return 'red';
};
