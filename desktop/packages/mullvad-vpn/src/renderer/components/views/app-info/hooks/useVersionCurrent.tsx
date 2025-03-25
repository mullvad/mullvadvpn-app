import { useSelector } from '../../../../redux/store';

export const useVersionCurrent = () => {
  return useSelector((state) => state.version.current);
};
