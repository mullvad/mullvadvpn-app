import { useSelector } from '../../../../redux/store';

export const useVersionIsBeta = () => {
  return useSelector((state) => state.version.isBeta);
};
