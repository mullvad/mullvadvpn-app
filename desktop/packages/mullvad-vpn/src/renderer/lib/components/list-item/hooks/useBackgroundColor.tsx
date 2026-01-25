import { levels } from '../levels';
import { useListItemContext } from '../ListItemContext';

export const useBackgroundColor = () => {
  const { level, disabled } = useListItemContext();
  return disabled ? levels[level].disabled : levels[level].enabled;
};
