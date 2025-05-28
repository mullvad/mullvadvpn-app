import { useVersionConsistent } from '../../../../../../redux/hooks';

export const useShowAlert = () => {
  const { consistent } = useVersionConsistent();
  return !consistent;
};
