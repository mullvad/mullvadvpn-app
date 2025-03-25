import { useVersionConsistent } from '../../../hooks';

export const useShowFooter = () => {
  const { consistent } = useVersionConsistent();
  return !consistent;
};
