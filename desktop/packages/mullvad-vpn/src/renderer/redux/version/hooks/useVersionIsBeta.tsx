import { useSelector } from '../../store';

export const useVersionIsBeta = () => {
  return { isBeta: useSelector((state) => state.version.isBeta) };
};
