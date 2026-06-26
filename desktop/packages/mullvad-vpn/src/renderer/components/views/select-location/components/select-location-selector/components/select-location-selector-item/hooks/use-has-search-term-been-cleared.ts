import { usePrevious } from '../../../../../../../../hooks';
import { useSelectLocationViewContext } from '../../../../../SelectLocationViewContext';

export function useHasSearchTermBeenCleared() {
  const { searchTerm } = useSelectLocationViewContext();

  const previousSearchTerm = usePrevious(searchTerm);
  const hasSelectLocationSearchBeenCleared = !searchTerm && !!previousSearchTerm;

  return hasSelectLocationSearchBeenCleared;
}
