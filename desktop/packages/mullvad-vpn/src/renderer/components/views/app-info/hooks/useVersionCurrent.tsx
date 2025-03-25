import { useSelector } from '../../../../redux/store';

export const useVersionCurrent = () => {
  return { current: useSelector((state) => state.version.current) };
};
