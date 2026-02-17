import { useSelectLocationViewContext } from '../../../SelectLocationViewContext';

export function useHasSearched() {
  const { searchTerm } = useSelectLocationViewContext();

  const hasSearched = searchTerm !== '';

  return hasSearched;
}
