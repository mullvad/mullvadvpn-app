import { useVersionConsistent } from '../../../../../../redux/hooks';

export const useShowFooter = () => {
  const { consistent } = useVersionConsistent();
  return !consistent;
};
