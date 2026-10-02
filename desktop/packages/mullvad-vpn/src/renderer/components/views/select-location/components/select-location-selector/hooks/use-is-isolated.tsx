import { useSelectLocationViewContext } from '../../../SelectLocationViewContext';

export function useIsIsolated() {
  const { isolatedItem } = useSelectLocationViewContext();

  return isolatedItem !== undefined;
}
