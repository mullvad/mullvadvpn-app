import { useSelector } from '../../store';

export const useConnectionIsBlocked = () => {
  return { isBlocked: useSelector((state) => state.connection.isBlocked) };
};
