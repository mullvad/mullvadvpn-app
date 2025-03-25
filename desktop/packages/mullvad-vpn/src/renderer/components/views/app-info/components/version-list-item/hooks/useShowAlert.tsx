import { useVersionConsistent } from '../../../hooks';

export const useShowAlert = () => {
  const { consistent } = useVersionConsistent();
  return !consistent;
};
