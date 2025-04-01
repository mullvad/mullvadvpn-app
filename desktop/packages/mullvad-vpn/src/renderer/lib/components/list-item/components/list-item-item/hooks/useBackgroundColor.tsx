import { levels } from '../../../levels';
import { useListItem } from '../../../ListItemContext';

export const useBackgroundColor = () => {
  const { level, disabled } = useListItem();
  return disabled ? levels[level].disabled : levels[level].enabled;
};
