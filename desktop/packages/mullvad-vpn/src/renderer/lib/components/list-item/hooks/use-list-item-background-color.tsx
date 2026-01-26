import { levels } from '../levels';
import { useListItemContext } from '../ListItemContext';

export const useListItemBackgroundColor = () => {
  const { level, disabled } = useListItemContext();
  return disabled ? levels[level].disabled : levels[level].enabled;
};
