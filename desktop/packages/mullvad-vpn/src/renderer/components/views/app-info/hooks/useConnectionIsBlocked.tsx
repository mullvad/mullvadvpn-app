import { useSelector } from '../../../../redux/store';

export const useConnectionIsBlocked = () => {
  return useSelector((state) => state.connection.isBlocked);
};
