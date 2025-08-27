import { levels } from '../../../levels';
import { useListItem } from '../../../ListItemContext';

export const useIndent = () => {
  const { level } = useListItem();
  return levels[level].indent;
};
