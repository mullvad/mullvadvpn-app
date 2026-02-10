import { useSelectLocationViewContext } from '../../../SelectLocationViewContext';

export function useHasSearched() {
  const { searchTerm } = useSelectLocationViewContext();

  if (searchTerm !== '') {
    return true;
  }

  return false;
}
