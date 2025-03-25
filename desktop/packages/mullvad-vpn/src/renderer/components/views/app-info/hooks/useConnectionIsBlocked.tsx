import { useSelector } from '../../../../redux/store';

export const useConnectionIsBlocked = () => {
  return { isBlocked: useSelector((state) => state.connection.isBlocked) };
};
