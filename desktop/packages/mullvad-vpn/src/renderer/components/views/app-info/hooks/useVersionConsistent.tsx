import { useSelector } from '../../../../redux/store';

export const useVersionConsistent = () => {
  return useSelector((state) => state.version.consistent);
};
