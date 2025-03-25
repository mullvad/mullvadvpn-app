import { useSelector } from '../../../../redux/store';

export const useVersionConsistent = () => {
  return { consistent: useSelector((state) => state.version.consistent) };
};
