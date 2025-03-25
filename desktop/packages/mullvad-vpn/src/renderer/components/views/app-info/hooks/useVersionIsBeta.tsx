import { useSelector } from '../../../../redux/store';

export const useVersionIsBeta = () => {
  return { isBeta: useSelector((state) => state.version.isBeta) };
};
