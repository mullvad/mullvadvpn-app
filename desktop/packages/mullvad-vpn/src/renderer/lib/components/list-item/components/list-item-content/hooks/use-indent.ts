import { levels } from '../../../levels';
import { useListItemContext } from '../../../ListItemContext';

export const useIndent = () => {
  const { level } = useListItemContext();
  return levels[level].indent;
};
