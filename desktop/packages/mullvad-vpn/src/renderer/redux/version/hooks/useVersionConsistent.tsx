import { useSelector } from '../../store';

export const useVersionConsistent = () => {
  return { consistent: useSelector((state) => state.version.consistent) };
};
