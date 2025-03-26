import { useSelector } from '../../store';

export const useVersionCurrent = () => {
  return { current: useSelector((state) => state.version.current) };
};
